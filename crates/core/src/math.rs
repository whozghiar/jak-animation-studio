//! Quaternions as glTF stores them: [x, y, z, w].

pub type Quat = [f32; 4];

pub fn mul(a: Quat, b: Quat) -> Quat {
  let [ax, ay, az, aw] = a;
  let [bx, by, bz, bw] = b;
  [
    aw * bx + ax * bw + ay * bz - az * by,
    aw * by - ax * bz + ay * bw + az * bx,
    aw * bz + ax * by - ay * bx + az * bw,
    aw * bw - ax * bx - ay * by - az * bz,
  ]
}

pub fn normalize(q: Quat) -> Quat {
  let len = q.iter().map(|c| c * c).sum::<f32>().sqrt();
  if len < 1e-12 {
    [0.0, 0.0, 0.0, 1.0]
  } else {
    q.map(|c| c / len)
  }
}

fn axis_angle(axis: usize, degrees: f32) -> Quat {
  let half = degrees.to_radians() / 2.0;
  let mut q = [0.0, 0.0, 0.0, half.cos()];
  q[axis] = half.sin();
  q
}

/// Blender's XYZ Euler mode: about X, then Y, then Z.
pub fn from_euler_xyz_degrees(e: [f32; 3]) -> Quat {
  mul(
    axis_angle(2, e[2]),
    mul(axis_angle(1, e[1]), axis_angle(0, e[0])),
  )
}

/// The rotation of a column-major 4x4 matrix (its scale divided out first).
pub fn from_matrix(m: &[f32]) -> Quat {
  let col = |c: usize| {
    let v = [m[c * 4], m[c * 4 + 1], m[c * 4 + 2]];
    let len = v.iter().map(|x| x * x).sum::<f32>().sqrt().max(1e-12);
    v.map(|x| x / len)
  };
  let (x, y, z) = (col(0), col(1), col(2));
  let (m00, m11, m22) = (x[0], y[1], z[2]);
  let trace = m00 + m11 + m22;
  let q = if trace > 0.0 {
    let s = (trace + 1.0).sqrt() * 2.0;
    [
      (y[2] - z[1]) / s,
      (z[0] - x[2]) / s,
      (x[1] - y[0]) / s,
      s / 4.0,
    ]
  } else if m00 > m11 && m00 > m22 {
    let s = (1.0 + m00 - m11 - m22).sqrt() * 2.0;
    [
      s / 4.0,
      (y[0] + x[1]) / s,
      (z[0] + x[2]) / s,
      (y[2] - z[1]) / s,
    ]
  } else if m11 > m22 {
    let s = (1.0 + m11 - m00 - m22).sqrt() * 2.0;
    [
      (y[0] + x[1]) / s,
      s / 4.0,
      (z[1] + y[2]) / s,
      (z[0] - x[2]) / s,
    ]
  } else {
    let s = (1.0 + m22 - m00 - m11).sqrt() * 2.0;
    [
      (z[0] + x[2]) / s,
      (z[1] + y[2]) / s,
      s / 4.0,
      (x[1] - y[0]) / s,
    ]
  };
  normalize(q)
}

/// The opposite rotation of a unit quaternion.
pub fn inverse(q: Quat) -> Quat {
  [-q[0], -q[1], -q[2], q[3]]
}

/// Rotates `v` by `q`.
pub fn rotate(q: Quat, v: [f32; 3]) -> [f32; 3] {
  let p = mul(mul(q, [v[0], v[1], v[2], 0.0]), inverse(q));
  [p[0], p[1], p[2]]
}

#[cfg(test)]
mod tests {
  use super::*;

  fn close(a: &[f32], b: &[f32]) -> bool {
    a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-4)
  }

  #[test]
  fn euler_and_matrix_agree() {
    let q = from_euler_xyz_degrees([0.0, 0.0, 90.0]);
    assert!(close(&rotate(q, [1.0, 0.0, 0.0]), &[0.0, 1.0, 0.0]));
    // X then Z: (0,1,0) -> (0,0,1) by X 90, stays (0,0,1) by Z 90
    let q = from_euler_xyz_degrees([90.0, 0.0, 90.0]);
    assert!(close(&rotate(q, [0.0, 1.0, 0.0]), &[0.0, 0.0, 1.0]));
    // a matrix scaled by 2 and turned 90 degrees about Y
    let m = [
      0.0, 0.0, -2.0, 0.0, 0.0, 2.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 1.0,
    ];
    let expected = from_euler_xyz_degrees([0.0, 90.0, 0.0]);
    let got = from_matrix(&m);
    assert!(close(&got, &expected) || close(&got.map(|c| -c), &expected));
  }
}
