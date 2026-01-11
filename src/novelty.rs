use rand::Rng;

/*
1. 初期集団の生成: ランダムに個体を生成し、初期集団を形成する。
    - ※空間の次元を n とする。
    - 各個体が持つべき情報
        - 空間座標（行動記述子）: n 次元ベクトル
        - 新規性スコア: 実数値（例: `f64`）
    - 空間（環境）が持つべき情報
        - アーカイブ: 過去に「新規」と判断された n 次元ベクトルの集合
*/
pub fn gen_population(
    agents: usize,
    dimensions: usize,
    random_min: f64,
    random_max: f64,
    rng_mut: &mut rand_chacha::ChaCha12Rng,
) -> Vec<Vec<f64>> {
    // 集団の初期化
    let mut population: Vec<Vec<f64>> = Vec::new();

    //* 各個体の生成 *//
    for _ in 0..agents {
        // agentの初期化
        let mut agent: Vec<f64> = Vec::new();

        // dimensionsの次元分のランダム座標を生成
        for _ in 0..dimensions {
            let point = rng_mut.random_range(random_min..=random_max); // random_min 〜 random_max のランダム座標
            agent.push(point);
        }
        population.push(agent);
    }

    population
}

/*
2. 評価関数の適用: 各個体 x に対して、
    - 現在の対象の個体の座標と、アーカイブおよび現在の集団の他の各個体の座標との距離を計算し、
    - その中から k 個の最近傍を取り、その平均距離を新規性スコアとする。
        - 距離の計算にはユークリッド距離などのメトリックを用いる。
        - このスコアが大きいほど（近くにある他の個体が少ないほど）、新規性が高いと判断される。

3. アーカイブの更新:
    - 新規性スコアがあらかじめ定めた閾値を超えた個体の座標を、アーカイブに追加する。
 */
fn calc_novelty_score(
    target_agent: &[f64],
    target_agent_index: usize,
    population: &[Vec<f64>],
    archive: &[Vec<f64>],
    k: usize,
) -> f64 {
    // ==== 現在の他の個体との距離を格納する集合 ====
    let mut distances: Vec<f64> = Vec::new();

    // 1. アーカイブとの距離
    // "アーカイブに保存されている全個体"の座標との距離を計算
    for archive_agent in archive {
        let d = distances::vectors::euclidean(target_agent, archive_agent);
        distances.push(d);
    }

    // 2. "現在の集団"の「他の個体」との距離
    // 全個体との距離を計算してリストに追加
    for (current_agent_index, current_agent) in population.iter().enumerate() {
        if target_agent_index == current_agent_index {
            // 自分自身はスキップ
            continue;
        } else {
            // 他の個体との距離を計算
            let d = distances::vectors::euclidean(target_agent, current_agent);
            distances.push(d);
        }
    }

    // 3. 現在評価中の個体との距離を小さい順にソート
    distances.sort_by(|a, b| a.partial_cmp(b).unwrap());

    // 4. 近い順に k 個を見て、その平均距離をスコアとする
    if distances.is_empty() {
        return 0.0;
    }
    let kk = k.min(distances.len());
    // ==== 他の現在評価中の個体に近い順に k 個を見たときのそれぞれの距離の合計を格納する変数 ====
    let kk_distance_sum = distances[0..kk].iter().sum::<f64>();

    // 新規性スコア（平均距離）
    kk_distance_sum / kk as f64
}

pub fn evaluate_novelty(
    population: &[Vec<f64>],
    archive: &mut Vec<Vec<f64>>,
    k: usize,
    threshold: f64,
) -> Vec<(Vec<f64>, f64)> {
    // ==== agentの点数順にソートした現世代のpopulation ====
    // (agent, novelty_score)
    let mut novelty_scores: Vec<(Vec<f64>, f64)> = Vec::new();

    for (agent_index, agent) in population.iter().enumerate() {
        // 新規性スコアの計算
        let novelty_score = calc_novelty_score(agent, agent_index, population, archive, k);

        // 5. 閾値を超えたらアーカイブに追加
        if novelty_score > threshold {
            archive.push(agent.clone());
        }

        // 6. 次世代個体群に (個体, 新規性スコア) の組を追加
        novelty_scores.push((agent.clone(), novelty_score));
    }

    // 新規性スコアの高い順にソート
    novelty_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    // (agent, novelty_score) の組を返す
    novelty_scores
}

/*
4. 選択:
    - 新規性スコアに基づいて、次世代に進む個体を選択する。
 */
pub fn select_novelty(scored_population: &[(Vec<f64>, f64)], alive_agents: usize) -> Vec<Vec<f64>> {
    // alive_agents文だけ新規性スコアの高い個体を選択
    scored_population
        .iter()
        .take(alive_agents)
        .map(|(agent, _score)| agent.clone())
        .collect()
}

/*
5. 交叉と突然変異:
    - 選択された個体を用いて交叉と突然変異を行い、新しい個体を生成する。
*/
pub fn replenish_novelty(
    selected_population: &[Vec<f64>],
    alive_agents: usize,
    killed_agents: usize,
    noise_min: f64,
    noise_max: f64,
    rng: &mut rand_chacha::ChaCha12Rng,
) -> Vec<Vec<f64>> {
    // ==== 生成された次世代個体群 ====
    let mut next_population: Vec<Vec<f64>> = Vec::new();
    // ==== 子供の個体 ====
    let mut child_agent: Vec<f64>;

    // === 交叉と突然変異 === //
    for _ in 0..killed_agents {
        // 交叉: ランダムに2個体を選んで平均を取る
        let parent1 = &selected_population[rng.random_range(0..alive_agents)];
        let parent2 = &selected_population[rng.random_range(0..alive_agents)];

        child_agent = Vec::new();

        for i in 0..parent1.len() {
            let mixed_agent = (parent1[i] + parent2[i]) / 2.0;
            child_agent.push(mixed_agent);
        }

        // 突然変異: 各次元に小さなランダムノイズを加える
        for dimension in &mut child_agent {
            let noise = rng.random_range(noise_min..=noise_max); // noise_min 〜 noise_max のランダムノイズ
            *dimension += noise;
        }

        next_population.push(child_agent);
    }

    next_population
}
