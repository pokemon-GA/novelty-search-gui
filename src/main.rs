use rand::SeedableRng;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::keyboard::Scancode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use std::time::Duration;
use std::time::Instant;

mod novelty;
use crate::novelty::{evaluate_novelty, gen_population, replenish_novelty, select_novelty};

pub fn main() {
    // sdl2の初期化
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let mut window = video_subsystem
        .window("Novelty Evaluation", 800, 600)
        .position_centered()
        .resizable()
        .build()
        .unwrap();
    // Fullscreenの切り替え用フラグ
    let mut is_fullscreen = false;
    // 最初のWindow状態は通常表示
    window
        .set_fullscreen(sdl2::video::FullscreenType::Off)
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    // カメラの初期化
    let mut cam_x: f64 = 0.0; // カメラの位置 x方向
    let mut cam_y: f64 = 0.0; // カメラの位置 y方向
    let mut vel_x: f64 = 0.0; // カメラの速度 x方向
    let mut vel_y: f64 = 0.0; // カメラの速度 y方向
    let accel: f64 = 700.0; // 加速度
    let damping_per_sec: f64 = 5.0; // 減衰率
    let max_speed: f64 = 800.0; // 最大速度
    let mut decay: f64; // 減衰計算用
    let mut last_instant = Instant::now(); // 前回のフレーム時間計測用

    // novelty searchの初期化
    let mut _generation: usize = 0; // 世代数カウンタ
    // ==== パラメータ ====
    let k: usize = 7; // 近傍として見る個体数
    let threshold: f64 = 0.5; // アーカイブ追加の閾値
    let agents: usize = 50; // 集団の個体数
    let alive_agents = 7; // 次世代に残す個体数
    let killed_agents = agents - alive_agents; // 次世代に進まない個体数
    let dimensions: usize = 2; // 空間の次元数
    let mut rng_init = rand_chacha::ChaCha12Rng::seed_from_u64(1); // 初期生成用のランダム生成器
    let mut rng_mut = rand_chacha::ChaCha12Rng::seed_from_u64(1); // 突然変異用のランダム生成器
    let random_min: f64 = 0.0; // 初期生成時のランダム座標の最小値
    let random_max: f64 = 1.0; // 初期生成時のランダム座標の最大値
    let noise_min: f64 = -0.6; // 突然変異ノイズの最小値
    let noise_max: f64 = 0.6; // 突然変異ノイズの最大値
    // ==== アーカイブ（最初は空）====
    let mut archive: Vec<Vec<f64>> = Vec::new();
    // ==== agentの点数順にソートした現世代のpopulation ====
    // (agent, novelty_score)
    let mut scored_population: Vec<(Vec<f64>, f64)>;
    // ==== 選択された次世代個体群 (alive_agentsの個体) ====
    let mut selected_population: Vec<Vec<f64>>;
    let remain_agents = 7; // 次世代に残す個体数
    // ==== 生成された次世代個体群 (killed_agentsで消えた個体の補填分の新しい子の個体) ====
    let mut next_population: Vec<Vec<f64>>;

    // ==== novelty searchの制御処理 ====
    // println!("--- Novelty Evaluation ---");

    // ==== 初期集団（2次元空間上の12点）====
    let mut population: Vec<Vec<f64>> =
        gen_population(agents, dimensions, random_min, random_max, &mut rng_init);

    'running: loop {
        // 背景
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        // ==== novelty searchの各世代の制御処理 ====
        // println!("\nGeneration {}", _generation);
        // println!("Archive size: {}", archive.len());
        // 世代数カウンタを進める
        // _generation += 1;

        // === 各個体について新規性の評価 === //
        // scored_population: Vec<(agent, novelty_score)>
        scored_population = evaluate_novelty(&population, &mut archive, k, threshold);

        // === 選択 (エリート選択) === //
        // remain_agents分だけ新規性スコアの高い個体を選択
        selected_population = select_novelty(&scored_population, remain_agents);

        // === 交叉と突然変異 === //
        next_population = replenish_novelty(
            &selected_population,
            alive_agents,
            killed_agents,
            noise_min,
            noise_max,
            &mut rng_mut,
        );

        // 次世代個体群を更新
        population.clear();
        population.extend(selected_population);
        population.extend(next_population);

        // ==== sdl2 の制御処理 ====
        // イベント処理
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    break 'running;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::F11),
                    ..
                } => {
                    is_fullscreen = !is_fullscreen;
                    canvas
                        .window_mut()
                        .set_fullscreen(if is_fullscreen {
                            sdl2::video::FullscreenType::Desktop
                        } else {
                            sdl2::video::FullscreenType::Off
                        })
                        .unwrap();
                }
                _ => {}
            }
        }

        // dt の計算
        let current_instant = Instant::now();
        let dt = current_instant.duration_since(last_instant).as_secs_f64();
        last_instant = current_instant;

        // カメラの移動速度の更新
        let keyboard_state = event_pump.keyboard_state();
        // キー入力による加速度の適用
        if keyboard_state.is_scancode_pressed(Scancode::W) {
            vel_y += accel * dt;
        }
        if keyboard_state.is_scancode_pressed(Scancode::S) {
            vel_y -= accel * dt;
        }
        if keyboard_state.is_scancode_pressed(Scancode::A) {
            vel_x += accel * dt;
        }
        if keyboard_state.is_scancode_pressed(Scancode::D) {
            vel_x -= accel * dt;
        }
        // 減衰力の適用
        if !(keyboard_state.is_scancode_pressed(Scancode::W)
            || keyboard_state.is_scancode_pressed(Scancode::S)
            || keyboard_state.is_scancode_pressed(Scancode::A)
            || keyboard_state.is_scancode_pressed(Scancode::D))
        {
            decay = (-damping_per_sec * dt).exp();
            vel_x *= decay;
            vel_y *= decay;
        }
        // 速度制限
        if vel_x > max_speed {
            vel_x = max_speed;
        }
        if vel_x < -max_speed {
            vel_x = -max_speed;
        }
        if vel_y > max_speed {
            vel_y = max_speed;
        }
        if vel_y < -max_speed {
            vel_y = -max_speed;
        }
        // 位置更新
        cam_x += vel_x * dt;
        cam_y += vel_y * dt;

        // ==== sdl2 の描画処理 ====
        // Windowのサイズを取得
        let (width, height) = canvas.output_size().unwrap();
        // 各個体を画面に描画 (archiveの各点を赤色の小さな四角で表示)
        // x: width/2, y: height/2 を中心座標とする
        for point in &archive {
            let x = (point[0] * 10.0) as i32 + width as i32 / 2 + cam_x as i32;
            let y = (point[1] * 10.0) as i32 + height as i32 / 2 + cam_y as i32;
            let rect = Rect::new(x, y, 1, 1);
            canvas.set_draw_color(Color::RGB(255, 0, 0));
            canvas.fill_rect(rect).unwrap();
        }

        // 各個体を画面に描画 (populationの各点を緑色の小さな四角で表示)
        // x: width/2, y: height/2 を中心座標とする
        for agent in &population {
            let x = (agent[0] * 10.0) as i32 + width as i32 / 2 + cam_x as i32;
            let y = (agent[1] * 10.0) as i32 + height as i32 / 2 + cam_y as i32;
            let rect = Rect::new(x, y, 1, 1);
            canvas.set_draw_color(Color::RGB(0, 255, 0));
            canvas.fill_rect(rect).unwrap();
        }

        // 画面に表示
        canvas.present();

        // 次のフレームが発火するまでの時間を計算
        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
