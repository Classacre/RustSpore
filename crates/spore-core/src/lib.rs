//! Engine-agnostic core types for RustSpore.
//!
//! This crate deliberately has no Bevy dependency. Anything that should be
//! testable without a running renderer, or reused by a headless tool, belongs
//! here. Gameplay snapshots use bounded, versioned binary serialization. Resource
//! parsers accept payload slices independently of the DBPF container format.

use std::fmt;

pub mod anim;
pub mod anim_curve;
pub mod anim_binding;
pub mod anim_body_pose;
pub mod anim_body_capabilities;
pub mod anim_body_finalizer_fields;
pub mod anim_cid_node_init_v1;
pub mod anim_cid_node_validity_v1;
pub mod anim_body_optional_feet;
pub mod anim_body_source;
pub mod anim_body_source_anchors;
pub mod anim_body_source_fields;
pub mod anim_body_topology;
pub mod anim_deform;
pub mod anim_order;
pub mod anim_nonroot_selector;
pub mod anim_nonroot_selector_z;
pub mod anim_ungrouped_binding_builder;
pub mod anim_instance_reference_handoff_v1;
pub mod anim_selector_z_bounds;
pub mod anim_palette;
pub mod anim_quaternion;
pub mod anim_rotation_curve;
pub mod anim_root_binding;
pub mod anim_root_pose;
pub mod anim_root_zero_child_stage;
pub mod audio_event;
pub mod bem;
pub mod cell;
pub mod cell_movement_short_part_step_v1;
pub mod cell_movement_short_part_flow_v1;
pub mod cell_indexed_input_vector_v1;
pub mod cell_registered_database_choice_v1;
pub mod cell_virtual_key_snapshot_v1;
pub mod cell_input_angle_wrap_v1;
pub mod cell_input_angle_vector_v1;
pub mod cell_legacy_ciasin_interior_v1;
pub mod cell_input_target_angle_v1;
pub mod cell_input_candidate_normalize_v1;
pub mod cell_input_target_early_v1;
pub mod cell_input_player_target_fixture_v1;
pub mod cell_capability_curve;
pub mod cell_capability_geometry;
pub mod cell_capability_scalar;
pub mod cell_definition;
pub mod cell_editor_template_array_v3;
pub mod creature_generated_skin_record_v1;
pub mod creature_generated_skin_ranking_v1;
pub mod creature_generated_skin_kernel_v1;
pub mod creature_generated_skin_grid_pack_v1;
pub mod creature_skin_grid_geometry_v1;
pub mod creature_skin_grid_retry_v1;
pub mod creature_skin_lattice_world_v1;
pub mod creature_skin_crossing_v1;
pub mod creature_skin_scalar_field_v1;
pub mod creature_skin_sse_field_v1;
pub mod creature_skin_sse2_field_v1;
pub mod creature_skin_class5_prefix_v1;
pub mod creature_skin_class5_segments_v1;
pub mod creature_skin_segment_zero_threshold_v1;
pub mod creature_skin_segment_crt_admitted_v1;
pub mod creature_generated_skin_grid_query_v1;
pub mod creature_generated_skin_vertex_fixture_v1;
pub mod creature_generated_skin_high_accum_v1;
pub mod creature_generated_skin_edge_accum_v1;
pub mod cell_progression_v1;
pub mod cell_progress_precall_v1;
pub mod cell_feeding_route_v2;
pub mod cell_feeding_callback_boundary_v2;
pub mod cell_feeding_contact_arbitration_e59b40_v1;
pub mod gmdl_skin_raw_influences_v1;
pub mod gmdl_direct_draw_influences_v1;
pub mod gmdl_direct_draw_position_diagnostic_v1;
pub mod gmdl_state4_upload_diagnostic_v1;
pub mod animlist_v10;
pub mod animlist_v7;
pub mod animlist_v8_v9;
pub mod cell_anim_state_table_v1;
pub mod cell_anim_source_crosslink_v1;
pub mod cell_request_weighted_suffix_v1;
pub mod cell_request_admission_projection_v1;
pub mod cell_request_snapshot_selection_v1;
pub mod cell_request_rng_v1;
pub mod cell_request_seeded_selection_v1;
pub mod cell_movement_animation_gate_v1;
pub mod cell_screen_quad_v1;
pub mod cell_world_population_view_v1;
pub mod cell_kind4_list_disk_v1;
pub mod cell_kind4_cache_selection_v1;
pub mod cell_kind4_integer_filter_v1;
pub mod cell_world_lighting_v1;
pub mod anim_cid_child_mode2_v1;
pub mod cell_effdir_framing_v3;
pub mod cell_skybox_inspection_v1;
pub mod effect_directory_startup_plan_v1;
pub mod cell_globals;
pub mod cell_flow_field;
pub mod cell_flow_aggregation_v1;
pub mod cell_flow_binning_v1;
pub mod cell_flow_snapshot_v1;
pub mod cell_movement;
pub mod cell_feeding_tail_e6a2b0_v1;
pub mod cell_feeding_producer_prefix_v1;
pub mod cell_player_target_bridge_v1;
pub mod cell_movement_application;
pub mod cell_movement_source;
pub mod cell_movement_scaling;
pub mod cell_part_geometry;
pub mod cell_part_classification;
pub mod cell_runtime_data;
pub mod cell_runtime_curves;
pub mod cell_runtime_modifiers;
pub mod cell_runtime_contributions;
pub mod cell_runtime_root_data;
pub mod cell_runtime_clock;
pub mod cell_runtime_clock_root_data;
pub mod cell_runtime_loop_clock;
pub mod cell_runtime_clip_metadata;
pub mod cell_runtime_source_clock_root_data;
pub mod cell_runtime_controller_loop;
pub mod cell_runtime_fresh_aid;
pub mod cell_runtime_fresh_loop;
pub mod cell_runtime_fresh_sampled_channel_v2;
pub mod cell_actor_pool_checked_v1;
pub mod cell_actor_geometry_projection_v1;
pub mod cell_broadphase_gate_v1;
pub mod cell_registration_prefix_v1;
pub mod cell_fresh_sweep_registration_v1;
pub mod cell_sweep_pair_filter_v1;
pub mod cell_sweep_endpoint_compare_v1;
pub mod cell_source_sweep_pairs_v1;
pub mod cell_start_equal_size_v1;
pub mod cell_start_null_orientation_v1;
pub mod cell_start_quaternion_matrix_v1;
pub mod cell_swimming;
pub mod civilization;
pub mod blockdata;
pub mod blockdata_query_v1;
pub mod blockdata_lookup_v1;
pub mod content_manifest;
pub mod creature_data;
pub mod creature_data_edit;
pub mod creature_data_session;
pub mod creature_game;
pub mod editor;
pub mod gmdl;
pub mod gmdl_skin;
pub mod gmsh;
pub mod hash_pairs;
pub mod mesh;
pub mod locale_table;
pub mod native_crt_acos;
pub mod native_sse;
pub mod cell_contact_impact_flag_v1;
pub mod native_x87;
pub mod material_links;
pub mod part_capabilities;
pub mod prop;
pub mod raster;
pub mod rw4;
pub mod rw4_animation;
pub mod shader_fragments;
pub mod shader_binding;
pub mod shader_codegen;
pub mod shader_pixel_codegen;
pub mod shader_pack;
pub mod shader_selection;
pub mod skin;
pub mod space;
pub mod spui;
pub mod spui_draw;
pub mod spui_effect;
pub mod spui_font;
pub mod spui_font_outline;
pub mod spui_font_scaling;
pub mod spui_font_scaling_quotient;
pub mod spui_font_hint_program;
pub mod spui_font_hint_control;
pub mod spui_font_hint_prep;
pub mod spui_font_hint_glyph0;
pub mod spui_font_raster_glyph0;
pub mod spui_font_hint_axis;
pub mod spui_font_raster_axis_rectangle;
pub mod spui_font_hint_rectilinear;
pub mod spui_font_raster_rectilinear;
pub mod spui_font_hint_simple_curved;
pub mod spui_font_hint_simple_loop;
pub mod spui_font_hint_simple_controls;
pub mod spui_font_hint_simple_state;
pub mod spui_font_hint_simple_cvt;
pub mod spui_font_vector_primitives;
pub mod spui_font_hint_general_positive;
pub mod spui_font_hint_general_intersection;
pub mod spui_font_hint_general_f20_state_v1;
pub mod spui_font_empty_caller_v1;
pub mod spui_font_raster_simple_contours_maxp_v1;
pub mod spui_font_intersection;
pub mod spui_font_raster_simple_contours;
pub mod spui_font_raster_simple_curved;
pub mod spui_glyph_atlas;
pub mod spui_layout;
pub mod spui_proportional_layout;
pub mod spui_std_drawable;
pub mod spui_std_drawable_mode2;
pub mod string_table;
pub mod tribe;

/// One of the five stages of SPORE, in progression order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum GameStage {
    /// The single cell stage played in a tide pool.
    Cell,
    /// The creature stage played on the surface of the home planet.
    Creature,
    /// The tribal stage.
    Tribe,
    /// The civilization stage.
    Civilization,
    /// The space stage.
    Space,
}

impl GameStage {
    /// Every stage, in progression order.
    pub const ALL: [GameStage; 5] = [
        GameStage::Cell,
        GameStage::Creature,
        GameStage::Tribe,
        GameStage::Civilization,
        GameStage::Space,
    ];

    /// The lowercase name of the stage.
    pub const fn name(self) -> &'static str {
        match self {
            GameStage::Cell => "cell",
            GameStage::Creature => "creature",
            GameStage::Tribe => "tribe",
            GameStage::Civilization => "civilization",
            GameStage::Space => "space",
        }
    }

    /// The next stage in progression, or `None` after the space stage.
    pub const fn next(self) -> Option<GameStage> {
        match self {
            GameStage::Cell => Some(GameStage::Creature),
            GameStage::Creature => Some(GameStage::Tribe),
            GameStage::Tribe => Some(GameStage::Civilization),
            GameStage::Civilization => Some(GameStage::Space),
            GameStage::Space => None,
        }
    }
}

impl fmt::Display for GameStage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// An opaque identifier for a game resource.
///
/// This is a RustSpore level handle. It is not the same thing as the DBPF
/// `ResourceKey` triple; conversion between the two is a later concern.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ResourceId(pub u64);

impl fmt::Display for ResourceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:016X}", self.0)
    }
}

/// A plain three component float vector.
///
/// This exists so engine-agnostic code can do simple maths without pulling in
/// Bevy. It is not a replacement for `bevy::math::Vec3` at the boundary.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const ZERO: Vec3 = Vec3::splat(0.0);
    pub const ONE: Vec3 = Vec3::splat(1.0);

    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub const fn splat(v: f32) -> Self {
        Self { x: v, y: v, z: v }
    }

    pub fn dot(self, other: Vec3) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn length(self) -> f32 {
        self.dot(self).sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stage_progression_is_ordered_and_terminates() {
        assert_eq!(GameStage::ALL.len(), 5);
        assert_eq!(GameStage::Cell.next(), Some(GameStage::Creature));
        assert_eq!(GameStage::Civilization.next(), Some(GameStage::Space));
        assert_eq!(GameStage::Space.next(), None);
    }

    #[test]
    fn stage_display_matches_name() {
        for stage in GameStage::ALL {
            assert_eq!(stage.to_string(), stage.name());
        }
    }

    #[test]
    fn resource_id_display_is_hex() {
        assert_eq!(ResourceId(0x1234).to_string(), "0000000000001234");
    }

    #[test]
    fn vec3_basic_maths() {
        let v = Vec3::new(3.0, 4.0, 0.0);
        assert_eq!(v.length(), 5.0);
        assert_eq!(v.dot(Vec3::new(1.0, 0.0, 0.0)), 3.0);
    }
}
pub mod snr_xas;

pub mod snr_pcm16;

pub mod eal31b;

pub mod cell_pick_tree_v1;
pub mod cell_pick_tree_build_v1;
pub mod cell_pick_filter_v1;
pub mod cell_start_table_v1;
pub mod spui_layout_propagation_v1;

pub mod creature_generated_skin_mixed_accum_v1;

pub mod creature_generated_skin_membership_v1;

pub mod creature_generated_skin_edges_v1;

pub mod creature_generated_skin_remap_v1;

pub mod creature_generated_skin_categories_v1;
pub mod cell_ui_progress_selection_v1;
pub mod cell_ui_progress_input_v1;
pub mod spui_window_draw_state_v1;
pub mod spui_window_listener_flags_v1;

pub mod creature_skin_corner_lookup_v1;

pub mod creature_skin_corner_sample_v1;
pub mod creature_skin_edge_lookup_v1;
pub mod creature_skin_edge_commit_v1;
pub mod creature_skin_edge_diagnostic_v1;
pub mod creature_skin_cube_walk_v1;
pub mod creature_skin_seed_v1;
pub mod creature_skin_queue_v1;
pub mod creature_skin_average_v1;
pub mod creature_skin_index_copy_v1;
pub mod creature_skin_gradient_v1;
pub mod cell_sweep_old_fresh_merge_v1;
pub mod creature_skin_normal_v1;
pub mod cell_sweep_two_frame_diagnostic_v1;
pub mod cell_sweep_pending_update_v1;
pub mod cell_sweep_remove_v1;
pub mod cell_sweep_sorted_crossing_v1;
pub mod cell_sweep_sorted_update_v1;
pub mod cell_sweep_multiframe_diagnostic_v1;
pub mod cell_actor_frame_update_v1;
pub mod cell_selector_route_v1;
pub mod cell_feature_loop_v1;
pub mod creature_skin_paired_topology_v1;
pub mod cell_feature_projection_v1;
pub mod creature_skin_sort64_v1;
pub mod creature_skin_smoothing_v1;
pub mod cell_feature_first_candidate_v1;
pub mod cell_feeding_b7_selection_v1;
pub mod creature_skin_corner_attributes_v1;
pub mod cell_e780_player_clear_v1;
pub mod cell_e780_root_unlink_v1;
pub mod cell_e780_manager_handoff_v1;
pub mod creature_skin_corner_triples_v1;
pub mod creature_skin_edge_continuity_v1;
pub mod creature_skin_face_orientation_v1;
pub mod cell_e590_component_release_v1;
pub mod cell_effect_pool_route_v1;
pub mod cell_effect_zero_handoff_v1;
pub mod cell_effect_placement_prefix_v1;
pub mod creature_skin_group_cachehit_v1;
pub mod cell_effect_curve_v1;
pub mod creature_skin_uv_select_v1;
pub mod creature_skin_uv_miss_v1;
pub mod creature_skin_group_seed_neighbors_v1;
pub mod creature_skin_group_queue_consumer_v1;
pub mod creature_skin_single_group_v1;
pub mod cell_effect_tier_v1;
pub mod cell_effect_existing_handoff_v1;
pub mod creature_skin_group_selector_v1;
pub mod creature_skin_multi_group_v1;
pub mod cell_ending_gate_v1;
pub mod cell_ending_effect_prefix_v1;

pub mod cell_transition_player_gate_v1;
pub mod cell_transition_event_gate_v1;
pub mod creature_skin_atlas_bounds_v1;
pub mod cell_mode_manager_prefix_v1;
pub mod creature_skin_shelf_append_v1;
pub mod cell_timeline_sender_v1;
pub mod cell_mode_direct_name_v1;
pub mod creature_skin_atlas_height_sort_v1;
pub mod cell_mode_generic_name_v1;
pub mod cell_timeline_receiver_v1;
pub mod creature_skin_shelf_select_v1;
pub mod cell_transition_record_gate_v1;
pub mod cell_transition_record_init_v1;
pub mod cell_service_action_gate_v1;
pub mod creature_skin_uv_transform_v1;
pub mod cell_pool_two_slot_v1;
pub mod cell_service_changed_prefix_v1;
pub mod creature_skin_atlas_offset_v1;
pub mod cell_record_timing_prefix_v1;
pub mod cell_record_update_helper_v1;
pub mod cell_record_timing_helper_join_v1;
pub mod creature_skin_atlas_pair_scale_v1;
pub mod cell_progress_callback_prefix_v1;
pub mod creature_skin_paired_offset_v1;
pub mod cell_fresh_save_constructor_v1;
pub mod cell_save_ownership_projection_v1;
pub mod cell_fresh_save_adoption_join_v1;
pub mod creature_skin_row_group_v1;
pub mod cell_save_restore_provider_v1;
pub mod cell_provider_cache_hit_v1;
pub mod cell_provider_registry_factory_v1;
pub mod cell_provider_two_lookup_join_v1;
pub mod creature_skin_uv_determinant_v1;
pub mod cell_provider_factory_validate_v1;
pub mod cell_provider_registry_factory_join_v1;
pub mod creature_skin_uv_tangent_prefix_v1;
pub mod cell_provider_descriptor_dispatch_v1;
pub mod creature_skin_uv_tangent_final_v1;
pub mod cell_provider_priority_scan_v1;
pub mod creature_skin_eight_group_triples_v1;
pub mod creature_skin_export_attributes_v1;
pub mod cell_provider_map_priority_join_v1;
pub mod creature_skin_export_indices_v1;
pub mod cell_provider_insert_lookup_join_v1;
pub mod cell_provider_cellsave_base_v1;
pub mod cell_provider_cellsave_key_v1;
pub mod cell_restore_wrapper_handoff_v1;
pub mod cell_restore_stream_gate_v1;
pub mod cell_stream_word_read_v1;
pub mod cell_camera_handoff_v1;
pub mod cell_camera_matrix_v1;
pub mod cell_camera_actor_branch_v1;
pub mod cell_camera_provider_join_v1;
pub mod cell_camera_continuation_v1;
pub mod cell_camera_provider_continuation_v1;
pub mod cell_camera_fullchain_v1;
pub mod cell_camera_earlier_vector_v1;
pub mod cell_camera_dispatch_v1;
pub mod cell_camera_component_provider_v1;
pub mod cell_camera_request_store_v1;
pub mod cell_camera_factory_v1;

pub mod cell_restore_writer_gate_v1;
pub mod cell_stream_path_gate_v1;
pub mod cell_stream_init_forward_v1;
pub mod cell_stream_open_prefix_v1;
pub mod cell_stream_capability_gate_v1;
pub mod cell_stream_length_dispatch_v1;
pub mod cell_stream_callback_dispatch_v1;
pub mod cell_stream_failure_continuation_v1;
pub mod cell_stream_close_projection_v1;
pub mod cell_stream_preopen_arguments_v1;
pub mod cell_stream_createfile_parameters_v1;
pub mod cell_stream_open_result_v1;
pub mod cell_effect_provider_fallback_v1;
pub mod cell_food_field_roundtrip_v1;
pub mod cell_effect_concrete_world_v1;
pub mod cell_food_writer_dispatch_v1;
pub mod cell_effect_map_lookup_v1;
pub mod cell_food_descriptor_order_v1;
pub mod cell_effect_map_insert_v1;
pub mod cell_food_full_writer_v1;
pub mod cell_food_full_roundtrip_v1;
pub mod cell_effect_transform_v1;
pub mod cell_start_bounds_v1;
pub mod cell_start_grid_v1;
pub mod cell_save_descriptor_order_v1;
pub mod cell_save_three_field_codec_v1;
pub mod cell_player_movement_v1;
pub mod cell_resource_index_v1;
pub mod civilization_vehicle_selector_v1;
pub mod tribe_selector_append_v1;
pub mod space_category_accessor_loop_v1;
pub mod descriptor_frame_v1;
pub mod space_locale_prefix_handoff_v1;
pub mod space_category_locale_projection_v1;
pub mod space_ui_handoff_v1;
pub mod planet_record_fields_v1;
pub mod space_ui_constructor_v1;
pub mod space_ui_lifecycle_v1;
pub mod civilization_first_entry_v1;
pub mod civilization_one_child_v1;
pub mod civilization_full_child_v1;
