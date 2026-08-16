use std::env;
use std::fs;
use std::path::PathBuf;

const W: u32 = 32;
const H: u32 = 32;

fn main() {
    let out_dir: PathBuf = env::var_os("OUT_DIR").expect("OUT_DIR").into();
    let rgba = render_rgba();
    fs::write(out_dir.join("icon.rgba"), &rgba).expect("write rgba");

    let target_windows =
        env::var("CARGO_CFG_TARGET_OS").map(|v| v == "windows").unwrap_or(false);
    if !target_windows {
        return;
    }

    let ico = build_ico(&rgba);
    let ico_path = out_dir.join("icon.ico");
    fs::write(&ico_path, &ico).expect("write ico");

    let rc_path = out_dir.join("app.rc");
    let ico_str = ico_path.to_string_lossy().replace('\\', "\\\\");
    fs::write(&rc_path, format!("app ICON \"{ico_str}\"\n")).expect("write rc");

    if let Err(e) =
        embed_resource::compile(&rc_path, embed_resource::NONE).manifest_optional()
    {
        println!("cargo:warning=icon embed failed: {e}");
    }

    println!("cargo:rerun-if-changed=build.rs");
}

fn render_rgba() -> Vec<u8> {
    let mut px = vec![0u8; (W * H * 4) as usize];
    let bg = [0x0D, 0x25, 0x57u8, 0xFF];
    let fg = [0xFF, 0xFF, 0xFF, 0xFF];

    for i in 0..(W * H) as usize {
        px[i * 4..i * 4 + 4].copy_from_slice(&bg);
    }

    let mut fill = |x0: u32, y0: u32, x1: u32, y1: u32| {
        for y in y0..y1 {
            for x in x0..x1 {
                if x < W && y < H {
                    let i = ((y * W + x) * 4) as usize;
                    px[i..i + 4].copy_from_slice(&fg);
                }
            }
        }
    };

    let top = 9;
    let bot = 23;

    let dx = 4;
    fill(dx, top, dx + 2, bot);
    fill(dx, top, dx + 7, top + 2);
    fill(dx, bot - 2, dx + 7, bot);
    fill(dx + 6, top + 2, dx + 8, bot - 2);

    let l1x = 14;
    fill(l1x, top, l1x + 2, bot);
    fill(l1x, bot - 2, l1x + 6, bot);

    let l2x = 22;
    fill(l2x, top, l2x + 2, bot);
    fill(l2x, bot - 2, l2x + 6, bot);

    px
}

fn build_ico(rgba: &[u8]) -> Vec<u8> {
    let mut bmp: Vec<u8> = Vec::with_capacity(40 + (W * H * 4) as usize + (W * H / 8) as usize);
    bmp.extend_from_slice(&40u32.to_le_bytes());
    bmp.extend_from_slice(&(W as i32).to_le_bytes());
    bmp.extend_from_slice(&((H * 2) as i32).to_le_bytes());
    bmp.extend_from_slice(&1u16.to_le_bytes());
    bmp.extend_from_slice(&32u16.to_le_bytes());
    bmp.extend_from_slice(&0u32.to_le_bytes());
    bmp.extend_from_slice(&0u32.to_le_bytes());
    bmp.extend_from_slice(&0i32.to_le_bytes());
    bmp.extend_from_slice(&0i32.to_le_bytes());
    bmp.extend_from_slice(&0u32.to_le_bytes());
    bmp.extend_from_slice(&0u32.to_le_bytes());

    for y in (0..H).rev() {
        for x in 0..W {
            let i = ((y * W + x) * 4) as usize;
            bmp.push(rgba[i + 2]);
            bmp.push(rgba[i + 1]);
            bmp.push(rgba[i]);
            bmp.push(rgba[i + 3]);
        }
    }
    bmp.extend(std::iter::repeat_n(0u8, (W * H / 8) as usize));

    let mut ico = Vec::with_capacity(22 + bmp.len());
    ico.extend_from_slice(&0u16.to_le_bytes());
    ico.extend_from_slice(&1u16.to_le_bytes());
    ico.extend_from_slice(&1u16.to_le_bytes());
    ico.push(W as u8);
    ico.push(H as u8);
    ico.push(0);
    ico.push(0);
    ico.extend_from_slice(&1u16.to_le_bytes());
    ico.extend_from_slice(&32u16.to_le_bytes());
    ico.extend_from_slice(&(bmp.len() as u32).to_le_bytes());
    ico.extend_from_slice(&22u32.to_le_bytes());
    ico.extend_from_slice(&bmp);
    ico
}
