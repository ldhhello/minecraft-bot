// 닉네임을 바탕으로 유니크한 색을 뽑아준다.
// ldh-monster-react에 있는 거랑 동일한 물건임
// 근데 결과가 좀 다르게 나오는데 이거는 왜그런지 잘 모르겠다

use colorutils_rs::{Oklch, TransferFunction};

fn hash(name: String) -> i32 {
    const MOD: i64 = 998244353;
    let mut res = 0;

    for i in 0..1000i64 {
        for ch in name.as_bytes().iter() {
            let xor= 153i64 ^ (i%256);
            let ascii = (*ch as i64) ^ xor;
            res = (res * 256 + ascii) % MOD;
        }
    }

    return res as i32;
}

pub fn get_unique_color(name: String) -> (u8, u8, u8) {
    let lightness = 0.6276;
    let chroma = 0.1488;
    let hue = f64::from(hash(name)) / 998244353.0 * 360.0 * 10.0 % 360.0;

    let rgb = Oklch::new(lightness, chroma, hue as f32).to_rgb(TransferFunction::Srgb);

    (rgb.r, rgb.g, rgb.b)
}