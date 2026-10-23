//! GPU validation without creating or controlling a desktop window.
use bevy::{
    app::ScheduleRunnerPlugin,
    camera::RenderTarget,
    prelude::*,
    render::{
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_asset::RenderAssets,
        render_resource::{CachedPipelineState, PipelineCache, TextureFormat, TextureUsages},
        texture::GpuImage,
        view::screenshot::Screenshot,
        Render, RenderApp, RenderSystems,
    },
    shader::PipelineCacheError,
    window::ExitCondition,
    winit::WinitPlugin,
};
use std::{
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};

#[derive(Resource)]
struct Size(u32, u32);
#[derive(Resource)]
pub(crate) struct OffscreenTarget(pub Handle<Image>);

/// Render-world feedback, rather than elapsed time, controls capture readiness.
/// Source previews can require particular images to have reached the GPU.
#[derive(Resource, Clone, ExtractResource, Default)]
pub(crate) struct OffscreenReadiness {
    shared: Arc<ReadinessState>,
    required_images: Vec<AssetId<Image>>,
}
#[derive(Default)]
struct ReadinessState {
    stable_frames: AtomicU32,
    failure: Mutex<Option<String>>,
}
impl OffscreenReadiness {
    pub fn require_image(&mut self, image: &Handle<Image>) {
        self.require_image_id(image.id());
    }
    fn require_image_id(&mut self, image: AssetId<Image>) {
        if !self.required_images.contains(&image) {
            self.required_images.push(image);
            self.shared.stable_frames.store(0, Ordering::Release);
        }
    }
    /// Text must have completed layout, and every glyph atlas must have reached
    /// the GPU. A compiled UI pipeline alone cannot prove fonts are ready.
    pub fn require_text(&mut self, layout: &bevy::text::TextLayoutInfo) -> bool {
        if layout.glyphs.is_empty() || !layout.size.is_finite() || layout.size.min_element() <= 0. {
            return false;
        }
        for glyph in &layout.glyphs {
            self.require_image_id(glyph.atlas_info.texture);
        }
        true
    }
    pub fn ready(&self) -> bool {
        self.shared.stable_frames.load(Ordering::Acquire) >= 8
    }
    pub fn failure(&self) -> Option<String> {
        self.shared.failure.lock().unwrap().clone()
    }
}
impl OffscreenTarget {
    pub fn screenshot(&self) -> Screenshot {
        Screenshot::image(self.0.clone())
    }
}
pub(crate) fn configure(app: &mut App, width: u32, height: u32) {
    assert!(width > 0 && height > 0 && width <= 8192 && height <= 8192);
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: None,
                exit_condition: ExitCondition::DontExit,
                ..default()
            })
            .disable::<WinitPlugin>(),
    )
    .add_plugins(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
        1. / 60.,
    )))
    .insert_resource(Size(width, height))
    .init_resource::<OffscreenReadiness>()
    .add_plugins(ExtractResourcePlugin::<OffscreenReadiness>::default())
    .add_systems(PreStartup, create_target)
    .add_systems(PostStartup, route_cameras);
    app.sub_app_mut(RenderApp)
        .add_systems(Render, report_readiness.in_set(RenderSystems::Cleanup));
}
fn create_target(
    mut commands: Commands,
    size: Res<Size>,
    mut images: ResMut<Assets<Image>>,
    mut readiness: ResMut<OffscreenReadiness>,
) {
    let mut image = Image::new_target_texture(size.0, size.1, TextureFormat::Rgba8UnormSrgb, None);
    image.texture_descriptor.usage |= TextureUsages::COPY_SRC;
    let target = images.add(image);
    readiness.require_image(&target);
    commands.insert_resource(OffscreenTarget(target));
}

fn report_readiness(
    readiness: Res<OffscreenReadiness>,
    pipelines: Res<PipelineCache>,
    images: Res<RenderAssets<GpuImage>>,
    mut previous_counts: Local<Option<(usize, usize)>>,
) {
    let count = pipelines.pipelines().count();
    let image_count = images.iter().count();
    let failure = pipelines
        .pipelines()
        .find_map(|pipeline| permanent_pipeline_error(&pipeline.state));
    let complete = failure.is_none()
        && count > 0
        && pipelines
            .pipelines()
            .all(|pipeline| matches!(pipeline.state, CachedPipelineState::Ok(_)))
        && pipelines.waiting_pipelines().next().is_none()
        && readiness
            .required_images
            .iter()
            .all(|id| images.get(*id).is_some());
    *readiness.shared.failure.lock().unwrap() = failure;
    let counts = (count, image_count);
    if complete && previous_counts.as_ref() == Some(&counts) {
        readiness
            .shared
            .stable_frames
            .fetch_add(1, Ordering::Release);
    } else {
        readiness.shared.stable_frames.store(0, Ordering::Release);
    }
    *previous_counts = Some(counts);
}

fn permanent_pipeline_error(state: &CachedPipelineState) -> Option<String> {
    match state {
        // Bevy explicitly retries these states when the shader/import arrives.
        // Treating them as terminal would abort normal cold-start compilation.
        CachedPipelineState::Err(
            PipelineCacheError::ShaderNotLoaded(_)
            | PipelineCacheError::ShaderImportNotYetAvailable,
        ) => None,
        CachedPipelineState::Err(error) => Some(error.to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unloaded_shader_and_import_wait_but_compilation_failure_remains_terminal() {
        assert!(permanent_pipeline_error(&CachedPipelineState::Queued).is_none());
        assert!(permanent_pipeline_error(&CachedPipelineState::Err(
            PipelineCacheError::ShaderNotLoaded(AssetId::invalid())
        ))
        .is_none());
        assert!(permanent_pipeline_error(&CachedPipelineState::Err(
            PipelineCacheError::ShaderImportNotYetAvailable
        ))
        .is_none());
        let error = permanent_pipeline_error(&CachedPipelineState::Err(
            PipelineCacheError::CreateShaderModule("invalid source shader".into()),
        ))
        .unwrap();
        assert!(error.contains("invalid source shader"));
    }
}
fn route_cameras(
    mut commands: Commands,
    target: Res<OffscreenTarget>,
    cameras: Query<(Entity, &Camera)>,
) {
    let default_camera = cameras
        .iter()
        .max_by_key(|(_, camera)| camera.order)
        .map(|(entity, _)| entity);
    for (entity, _) in &cameras {
        commands
            .entity(entity)
            .insert(RenderTarget::Image(target.0.clone().into()));
    }
    if let Some(entity) = default_camera {
        commands.entity(entity).insert(IsDefaultUiCamera);
    }
}
