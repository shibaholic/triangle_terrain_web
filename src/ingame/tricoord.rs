use bevy::reflect::{FromReflect, Reflect};

#[derive(Debug, Reflect)]
pub struct Coord<T> {
    pub z: T,
    pub x: T
}

#[derive(Copy, Clone, Debug, Reflect, PartialEq, Hash)]
pub struct TriCoord<T> {
    pub a: T,
    pub b: T,
    pub c: T
}

impl<T: Eq> Eq for TriCoord<T> {
}

pub const TRI_SIDE:f32 = 1.0;
pub const TRI_HALFSIDE:f32 = TRI_SIDE/2.0;
pub const TRI_ALTITUDE:f32 = 0.866025404;
pub const TRI_HALF_ALT:f32 = 0.433012702;
pub const TRI_APOTHEM:f32 = 0.288675135;

pub const CHUNK_SIDE:i16 = 16;
pub const CHUNK_HALFSIDE:f64 = (CHUNK_SIDE/2) as f64;
pub const CHUNK_ALTITUDE:f64 = 13.856406461;
pub const CHUNK_HALFALT:f64 = 6.92820323;
pub const CHUNK_APOTHEM:f64 = 4.618802154;

// basically going reverse: from the triangle find how many steps to get to origin.
pub fn trichunk_to_coord(tricoord: TriCoord<i16>, mode: u8) -> Coord<f64> {
    // converts a,b,c origin of trichunk to z,x world coordinates
    let mut temp = tricoord;
    let mut zx_coord = Coord::<f64> {z:0.0, x:0.0};

    // mesh center mode
    if mode == 1 {
        zx_coord = Coord::<f64> {z: TRI_APOTHEM as f64, x: (TRI_SIDE/2.0) as f64};
    }

    // world center mode
    if mode == 2 {
        zx_coord = Coord::<f64> {z: CHUNK_HALFALT as f64, x: CHUNK_HALFSIDE as f64};
    }

    // go to a == 0
    //println!("step 1");
    zx_coord.x += -temp.a as f64 * CHUNK_SIDE as f64;
    //println!("  x is now = {:.02}", zx_coord.x);
    temp.c = temp.c + temp.a;
    temp.a = 0;
    //println!("  a is now = {}, c is now = {}", temp.a, temp.c);

    // if odd then go to even
    //println!("step 2");
    let c_b_sum = temp.b + temp.c; // 1 if odd, 0 if even
    //println!("  c-b sum = {}", c_b_sum);

    //let s2_z_value = CHUNK_ALTITUDE/2.0 * c_b_sum as f64;
    //zx_coord.z += s2_z_value;
    //println!("  z change was {}, z is now = {}", s2_z_value, zx_coord.z);

    let s2_x_value = CHUNK_HALFSIDE * c_b_sum as f64;
    zx_coord.x += s2_x_value;
    //println!("  x change was {}, x is now = {}", s2_x_value, zx_coord.x);

    temp.c -= c_b_sum;
    //println!("  c is now {}", temp.c);

    // now b and c's absolute values equal each other
    // go to b == 0, c == 0
    //println!("step 3");
    //println!("  b is currently = {}", temp.b);
    let s3_z_value = temp.b as f64 * CHUNK_ALTITUDE;
    zx_coord.z += s3_z_value;
    //println!("  z change was {}, z is now = {}", s3_z_value, zx_coord.z);
    let s3_x_value = -temp.b as f64 * CHUNK_HALFSIDE;
    zx_coord.x += s3_x_value;
    //println!("  x change was {}, x is now = {}", s3_x_value, zx_coord.x);
    
    return zx_coord;
}

pub fn triangular_number_o1(n: i16) -> i16 {
    return n * (n + 1) / 2;
}

pub fn tricoord_vec_gen_distance(point:Coord<f32>, distance:f32) -> Vec<TriCoord<i16>> {

    let mut v:Vec<TriCoord<i16>> = Vec::new();

    // how many half_sides it takes to get to left most point that is within the distance from the point
    let leftmost_half_side = ((point.x - distance) / (CHUNK_SIDE as f32 / 2.0)) as i32; 
    let rightmost_half_side = ((point.x + distance) / (CHUNK_SIDE as f32 / 2.0)) as i32 + ((CHUNK_SIDE as f32 / 2.0)) as i32; 

    let botmost_altitude = ((point.z - distance) / (CHUNK_ALTITUDE as f32)) as i32; 
    let topmost_altitude = ((point.z + distance) / (CHUNK_ALTITUDE as f32)) as i32 + (CHUNK_ALTITUDE as i32); 

    for half_side_index in leftmost_half_side..rightmost_half_side {
        for altitude_index in botmost_altitude..topmost_altitude {
            let x = half_side_index as f32 * (CHUNK_SIDE as f32 / 2.0);
            let z = altitude_index as f32 * (CHUNK_ALTITUDE as f32);

            if f32::powi(x-point.x, 2) + f32::powi(z-point.z, 2) <= f32::powi(distance as f32, 2) {
                
                v.push(halfsides_altitude_to_tricoord(half_side_index, altitude_index));
            }
        }
    }

    return v;
}

// how do i convert from half_side, b to tricoord(a,b,c) ?
pub fn halfsides_altitude_to_tricoord(halfsides:i32, altitudes:i32) -> TriCoord<i16> {
    // odd is xor of halfsides and altitudes being odd.
    let odd = halfsides % 2 ^ altitudes % 2;

    let b = altitudes;

    let (mut a, mut c) = if halfsides < 0 {
        // negative halfsides = positive a and negative c
        let a = i32::abs(halfsides) % 2 + halfsides / 2 * -1;
        let c = halfsides / 2;
        (a,c) 
    } else {
        // positive halfsides = negative a and positive c
        let a = halfsides / 2 * -1;
        let c = i32::abs(halfsides) % 2 + halfsides / 2;
        (a,c)
    };

    let current_is_odd = i32::abs(halfsides) % 2;
    let ac_adjustment_value = if b < 0 {
        // negative altitudes
        (i32::abs(altitudes) + (current_is_odd^1) ) / 2
    } else if b > 0 {
        // positive altitudes
        (altitudes + current_is_odd ) / 2 * -1 
    } else {
        0
    };

    a += ac_adjustment_value;
    c += ac_adjustment_value;

    return TriCoord { a:a as i16, b:b as i16, c:c as i16 };
}