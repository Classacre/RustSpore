//! Bounded projection of 76F930. Arrays are explicitly supplied; no runtime association is implied.
//! Host admission requires A parents before children and acyclic B parents.
#[derive(Debug, PartialEq)]
pub enum Invalid { Length, Parent, Mapping }

pub fn check(a: &[i32], b: &[i32], map: &[i32]) -> Result<bool, Invalid> {
    if a.len()!=map.len() || a.len()>4096 || b.len()>4096 { return Err(Invalid::Length); }
    for (i,&p) in a.iter().enumerate() {
        if p < -1 || (p >= 0 && p as usize >= i) { return Err(Invalid::Parent); }
    }
    if b.iter().any(|&p| p < -1 || (p >= 0 && p as usize >= b.len())) { return Err(Invalid::Parent); }
    for i in 0..b.len() {
        let mut p = i as i32;
        let mut steps = 0usize;
        while p != -1 {
            steps += 1;
            if steps > b.len() { return Err(Invalid::Parent); }
            p = b[p as usize];
        }
    }
    if map.iter().any(|&m| m < -1 || (m >= 0 && m as usize >= b.len())) { return Err(Invalid::Mapping); }
    let mut any=false;
    for (i,&m) in map.iter().enumerate() {
        if m < 0 { continue; }
        any=true;
        let mut p=a[i];
        if p == -1 {
            if b[m as usize] != -1 { return Ok(false); }
        } else {
            while p != -1 && map[p as usize] == -1 { p=a[p as usize]; }
            if p == -1 || map[p as usize] != b[m as usize] { return Ok(false); }
        }
    }
    Ok(any)
}

fn main() {
    let cases: &[(&[i32],&[i32],&[i32])] = &[
        (&[],&[],&[]), (&[-1],&[-1],&[-1]), (&[-1],&[-1],&[0]),
        (&[-1,0,1],&[-1,0],&[0,-1,1]), (&[-1,0,1],&[-1,-1],&[0,-1,1]),
        (&[-1,0],&[-1,0],&[-1,1]), (&[-1,0],&[-1,0],&[1,0]),
        (&[-1,0,1],&[-1,0,1],&[0,1,2]),
    ];
    for (a,b,m) in cases { println!("{}",u8::from(check(a,b,m).unwrap())); }
}
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn bounded_cases() {
        assert_eq!(check(&[],&[],&[]),Ok(false));
        assert_eq!(check(&[-1,0,1],&[-1,0],&[0,-1,1]),Ok(true));
        assert_eq!(check(&[-1,0,1],&[-1,-1],&[0,-1,1]),Ok(false));
        assert_eq!(check(&[-1,0],&[-1,0],&[-1,1]),Ok(false));
        assert_eq!(check(&[-1],&[-1],&[0]),Ok(true));
        assert_eq!(check(&[-1],&[-1],&[-1]),Ok(false));
        assert_eq!(check(&[0],&[-1],&[0]),Err(Invalid::Parent));
        assert_eq!(check(&[-1],&[-1],&[-2]),Err(Invalid::Mapping));
        assert_eq!(check(&[-1],&[1,0],&[0]),Err(Invalid::Parent));
        assert_eq!(check(&[-1],&[1],&[0]),Err(Invalid::Parent));
        assert_eq!(check(&[-1],&[-1],&[1]),Err(Invalid::Mapping));
        assert_eq!(check(&[-1,2],&[-1],&[0,0]),Err(Invalid::Parent));
        assert_eq!(check(&[-1],&[-1],&[]),Err(Invalid::Length));
        assert_eq!(check(&[-1,-1],&[-1],&[0,0]),Ok(true));
        assert_eq!(check(&[-1,0],&[-1],&[-1,0]),Ok(false));
    }
}
