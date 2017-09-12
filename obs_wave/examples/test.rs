extern crate image;

use image::{Rgba, RgbaImage};
use std::process::Command;

fn main() {
    let mut model = Model::new();
    model.set(SIZE/2, SIZE/2, 1.0);

    for i in 0..1000 {
        let file = format!("assets/test-{}.png", i);
        model.to_image().save(&file).unwrap();

        model.update();
        model.update();
    }

    Command::new("ffmpeg")
        .arg("-r")
        .arg("4")
        .arg("-i")
        .arg("assets/test-%d.png")
        .arg("-pix_fmt")
        .arg("yuv444p")
        .arg("-y")
        .arg("test.mp4")
        .status()
        .expect("Failed to run command");
}

const SIZE: usize = 80;

type Grid = [[f32; SIZE]; SIZE];
type Field = [[[f32; 2]; SIZE]; SIZE];

pub struct Model {
    grid: Grid,
    new_grid: Grid,
    swap: bool,
    field: Field,
    phase: [f32; 2],
    phase_vel: [f32; 2],
}

impl Model {
    pub fn new() -> Model {
        use std::f32::consts::PI;

        // 45 degrees.
        let angle = 0.45;
        let phase_rad = 2.0 * PI * angle / 360.0;
        Model {
            grid: [[0.0; SIZE]; SIZE],
            new_grid: [[0.0; SIZE]; SIZE],
            swap: false,
            field: [[[0.0; 2]; SIZE]; SIZE],
            phase: [1.0, 0.0],
            phase_vel: [phase_rad.cos(), phase_rad.sin()],
        }
    }

    pub fn set(&mut self, x: usize, y: usize, prob: f32) {
        self.grid[y][x] = prob;
    }

    pub fn update(&mut self) {
        {
            let (a, b) = if self.swap {
                (&self.new_grid, &mut self.grid)
            } else {
                (&self.grid, &mut self.new_grid)
            };
            let ws = [
                1.0, 1.0, 1.0, 1.0
            ];
            for y in 0..SIZE {
                for x in 0..SIZE {
                    b[y][x] = (
                        ws[0] * a[y][(x+1)%SIZE] +
                        ws[1] * a[y][(x+SIZE-1)%SIZE] +
                        ws[2] * a[(y+1)%SIZE][x] +
                        ws[3] * a[(y+SIZE-1)%SIZE][x]
                    ) / 4.0;
                    self.field[y][x] = add(scale(self.field[y][x], 1.0 / 4.0),
                        scale(self.phase, b[y][x])
                    );
                }
            }
        }

        self.swap = !self.swap;
        self.phase = mul(self.phase, self.phase_vel);
    }

    pub fn to_image(&self) -> RgbaImage {
        let scale = 2;
        let mut image = RgbaImage::new(SIZE as u32 * scale, SIZE as u32 * scale);

        let grid = if self.swap {&self.new_grid} else {&self.grid};

        let f = |x: usize, y: usize| -> f32 {square_len(self.field[y][x])};
        let g = |x: usize, y: usize, sum: f32| -> f32 {
            f(x, y) / sum - grid[y][x]
        };

        let mut sum = 0.0;
        let mut max = 0.0;
        let mut max_grid = 0.0;
        for y in 0..SIZE {
            for x in 0..SIZE {
                let amp = f(x, y);
                sum += amp;
                if amp > max {max = amp;}
                if grid[y][x] > max_grid {max_grid = grid[y][x];}
            }
        }

        for y in 0..SIZE {
            for x in 0..SIZE {
                let diff = g(x, y, sum);
                let val_r = (clamp(0.05 * diff / max) * 255.0) as u8;
                let val_g = (clamp(0.05 * -diff / max) * 255.0) as u8;
                // let val_b = 255 - (clamp(f(x, y) / sum) * 255.0) as u8;
                // let val_r = val_g;
                let val_b = 0;
                for sy in 0..scale {
                    for sx in 0..scale {
                        image.put_pixel(x as u32 * scale + sx, y as u32 * scale + sy,
                            Rgba([val_r, val_g, val_b, 255]))
                    }
                }
            }
        }
        image
    }
}

fn clamp(val: f32) -> f32 {
    if val > 1.0 {1.0} else if val < 0.0 {0.0} else {val}
}

fn mul(a: [f32; 2], b: [f32; 2]) -> [f32; 2] {
    [a[0] * b[0] - a[1] * b[1], a[0] * b[1] + a[1] * b[0]]
}

fn scale(a: [f32; 2], s: f32) -> [f32; 2] {
    [a[0] * s, a[1] * s]
}

fn add(a: [f32; 2], b: [f32; 2]) -> [f32; 2] {
    [a[0] + b[0], a[1] + b[1]]
}

fn square_len(a: [f32; 2]) -> f32 {
    a[0] * a[0] + a[1] * a[1]
}
