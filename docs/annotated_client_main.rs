// ===================================================================
// client/src/main.rs - 詳細注釈版（Rust学習用）
// ===================================================================
// このファイルはクライアントアプリケーションのエントリーポイントです。
// PC情報を収集し、定期的にサーバーに送信する処理を実装しています。
// ===================================================================

// --- モジュール宣言 ---
// Rustのmodキーワードで、プロジェクト内の他のファイル（モジュール）を読み込みます
mod api;       // api/mod.rsまたはapi.rsを読み込む
mod config;    // config/mod.rsまたはconfig.rsを読み込む
mod error;     // エラー型定義モジュール
mod network;   // ネットワーク情報検出モジュール
mod wmi;       // WMI情報収集モジュール

// --- use文（インポート） ---
// use文で、他のモジュールや外部クレートの型・関数を現在のスコープに持ち込みます
// これにより、長い名前を省略できます（例: api::ApiClient → ApiClient）

use api::{ApiClient, PcInfoData};  // apiモジュールからApiClientとPcInfoData型をインポート
use config::ClientConfig;          // configモジュールからClientConfig型をインポート
use network::NetworkDetector;      // networkモジュールからNetworkDetector型をインポート
use wmi::WmiCollector;             // wmiモジュールからWmiCollector型をインポート
use chrono::Utc;                   // chronoクレートからUtc型（UTC日時）をインポート
use tokio::time::{interval, sleep, Duration}; // tokioの時間関連ユーティリティ
                                   // interval: 定期実行タイマー
                                   // sleep: 非同期スリープ
                                   // Duration: 期間を表す型
use tracing::{info, warn, error, debug}; // tracingクレートのログマクロ
                                   // info!: 情報ログ
                                   // warn!: 警告ログ
                                   // error!: エラーログ
                                   // debug!: デバッグログ
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
                                   // ログの購読者（subscriber）を構築するためのトレイト
use std::sync::Arc;                // Arc: Atomic Reference Counted
                                   // 複数のスレッド間で安全にデータを共有するためのスマートポインタ
                                   // 参照カウント方式で、すべての参照が消えたらメモリ解放
use tokio::sync::Mutex;            // tokio::sync::Mutex: 非同期対応のミューテックス
                                   // std::sync::Mutexと異なり、async/awaitで使える

// --- Enum定義：リトライ状態 ---
/// リトライ状態を表すenum（列挙型）
///
/// Rustのenumは、いくつかの異なる値（バリアント）のうち1つを持つ型です。
/// この場合、FirstRetryかSecondRetryのどちらかの状態を表します。
#[derive(Debug, Clone, Copy)]  // derive属性：自動的にトレイトを実装
                                // Debug: {:?}でフォーマット出力可能にする
                                // Clone: .clone()でコピー可能にする
                                // Copy: 代入時に暗黙的にコピーされる（軽量な型のみ）
enum RetryState {
    FirstRetry,     // 1回目のリトライ状態（15分待機）
    SecondRetry,    // 2回目のリトライ状態（1時間待機）
}

// --- main関数 ---
/// アプリケーションのエントリーポイント
///
/// #[tokio::main]属性マクロ：
/// - このmain関数をtokioの非同期ランタイム上で実行します
/// - 通常のmainはasyncにできませんが、このマクロで非同期main関数を書けます
/// - 実際には、マクロ展開で通常のmainがtokio runtimeをセットアップします
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 戻り値型の説明：
    // Result<T, E>: 成功時はOk(T)、失敗時はErr(E)を返す型
    // ここでは、Ok(()) が成功（()はユニット型、値なし）
    //         Err(Box<dyn std::error::Error>) がエラー
    // Box<dyn ...>: ヒープに確保されたトレイトオブジェクト
    // dyn std::error::Error: 任意のエラー型を格納できる

    // --- 設定ファイルパスの取得 ---
    // std::env::var: 環境変数を取得する関数
    // Result<String, VarError>を返す
    // unwrap_or_else: Err時にクロージャを実行してデフォルト値を返す
    // |_|: クロージャの引数（使わないので_）
    let config_path = std::env::var("CONFIG_PATH")
        .unwrap_or_else(|_| "config.toml".to_string());
    // 結果: 環境変数CONFIG_PATHがあればその値、なければ"config.toml"

    // --- 設定ファイルの読み込み ---
    // mut: mutable（可変）の略。後で変更可能にする
    // Rustはデフォルトでimmutable（不変）
    // ?演算子: Result型のErr時に早期リターン（関数から抜ける）
    //         Ok時は中身の値を取り出す
    let mut config = ClientConfig::load(&config_path)?;
    // &config_path: config_pathへの参照（borrowing）
    // 所有権を渡さず、読み取り専用で貸し出す
    // Rustの所有権システムにより、データ競合を防ぐ

    // --- ログシステムの初期化 ---
    // &config: configへの不変参照を渡す
    init_logging(&config);

    // --- ログ出力 ---
    // info!マクロ: フォーマット文字列でログ出力
    // {}プレースホルダーに値を埋め込む（std::fmt::Displayトレイト）
    info!("PC Inventory Client starting...");
    info!("Configuration loaded from: {}", config_path);
    info!("Server URL: {}", config.server.url);
    // .server.url: 構造体のフィールドアクセス
    // Rustでは、&configでもconfig.serverでアクセス可能（自動参照外し）

    // --- ユーザー名のバリデーション ---
    // if文: 条件分岐（Rustのifは式なので値を返せる）
    // .is_empty(): Stringの空チェックメソッド
    if config.pc_info.user_name.is_empty() {
        // ユーザー名が空の場合はエラーで終了
        error!("User name is not set in config.toml");
        error!("Please set your name in the [pc_info] section and restart the client");

        // return: 関数からの早期リターン
        // Err(...): Resultのエラーバリアント
        // .into(): From/Intoトレイトで型変換
        // "文字列".into() → Box<dyn std::error::Error>に変換
        return Err("User name is required".into());
    }

    // --- リトライ中フラグの初期化 ---
    // Arc::new: 新しいArcインスタンスを作成（ヒープ確保）
    // Mutex::new: 新しいMutexを作成（中身をロックで保護）
    // Arc<Mutex<bool>>の意味:
    //   - bool: リトライ中かどうかのフラグ
    //   - Mutex: 排他制御（一度に1つのタスクのみアクセス可能）
    //   - Arc: 複数のタスク間で共有（参照カウント方式）
    let is_retrying = Arc::new(Mutex::new(false));
    // 初期値はfalse（リトライしていない）

    // --- 起動時処理の実行 ---
    // if let パターンマッチ: Err時のみ処理を実行
    // Err(e): エラー値をeに束縛（bind）
    // await: 非同期関数の完了を待つ（async/awaitパターン）
    //        この間、他のタスクが実行可能（協調的マルチタスク）
    if let Err(e) = initial_process(&mut config, &config_path).await {
        // &mut config: 可変参照を渡す（関数内で変更可能）

        error!("Initial process failed: {}", e);

        // リトライサイクルを開始
        // .clone(): Arcの参照カウントを増やす（実際のデータはコピーしない）
        // config_path.clone(): Stringのクローン（ヒープ上の文字列をコピー）
        start_retry_cycle(is_retrying.clone(), config_path.clone()).await;
        // 引数にclone()を使う理由: 所有権を関数に移すため
        // Rustでは、値を渡すとmoveされる（元の変数は使えなくなる）
        // cloneで新しいコピーを作り、それを渡す
    }

    // --- 定期チェックタイマーの起動 ---
    info!("Starting periodic check timer (interval: {}s)", config.client.check_interval_secs);

    // interval関数: 指定した間隔で発火するタイマーを作成
    // Duration::from_secs: 秒数からDuration型を作成
    let mut check_timer = interval(Duration::from_secs(config.client.check_interval_secs));
    // mutが必要な理由: tick()メソッドが&mut selfを要求するため

    // --- メインループ ---
    // loop: 無限ループ（breakで抜けるまで繰り返す）
    loop {
        // --- タイマーの発火を待つ ---
        // tick(): 次のタイマー発火まで待機（awaitポイント）
        // 最初のtick()は即座に完了し、2回目以降は間隔ごとに発火
        check_timer.tick().await;
        debug!("Periodic check triggered");

        // --- 設定ファイルの再読み込み ---
        // match式: パターンマッチング（Rustの強力な制御構文）
        // Ok(new_config)パターンとErr(e)パターンに分岐
        match ClientConfig::load(&config_path) {
            // Okパターン: 成功時
            Ok(new_config) => {
                // new_configに束縛された値を使う
                config = new_config;  // 古いconfigを新しいもので上書き
                debug!("Configuration reloaded");
            }
            // Errパターン: エラー時
            Err(e) => {
                warn!("Failed to reload configuration: {}", e);
                // エラーでも処理は継続（既存の設定を使う）
            }
        }

        // --- 送信が必要かチェック ---
        // should_send: 非同期関数を呼び出し（await）
        // &config: configへの不変参照
        if should_send(&config).await {
            info!("Send interval elapsed, sending PC info");

            // periodic_check: 定期チェック処理を実行
            if let Err(e) = periodic_check(&mut config, &config_path).await {
                error!("Periodic check failed: {}", e);
                // 失敗時はリトライサイクル開始
                start_retry_cycle(is_retrying.clone(), config_path.clone()).await;
            }
        } else {
            debug!("Send interval not elapsed yet, skipping");
        }
        // loopの最後に到達 → 次のiteration（繰り返し）へ
    }
    // このコードは無限ループなので、ここには到達しない
    // プロセス終了はSIGTERMなどのシグナルで行う
}

// ===================================================================
// ログ初期化関数
// ===================================================================
/// ログシステムを初期化する関数
///
/// tracing-subscriberを使ってログ出力先を設定します。
/// - ファイル出力（client.log）
/// - コンソール出力（stdout）
///
/// # 引数
/// * `config` - クライアント設定への参照
fn init_logging(config: &ClientConfig) {
    // 引数: &ClientConfig - 不変参照（所有権を奪わない）
    // 戻り値: なし（()ユニット型）

    // --- ファイルアペンダーの作成 ---
    // tracing_appender::rolling::never: ログローテーションなし
    // 第1引数: ディレクトリ（"."はカレントディレクトリ）
    // 第2引数: ファイル名（config.logging.fileから取得）
    let file_appender = tracing_appender::rolling::never(".", &config.logging.file);

    // --- トレーシング購読者の構築 ---
    // registry(): ベースとなる購読者を作成
    tracing_subscriber::registry()
        // --- EnvFilterレイヤーを追加 ---
        // ログレベルフィルタリングを設定
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                // 環境変数RUST_LOGからフィルタを読み込む試み
                .unwrap_or_else(|_| config.logging.level.clone().into()),
                // 環境変数がなければconfig.tomlのlog levelを使用
                // .clone(): Stringをクローン
                // .into(): Stringから EnvFilterに変換
        )
        // --- ファイル出力レイヤーを追加 ---
        .with(tracing_subscriber::fmt::layer().with_writer(file_appender))
        // fmt::layer(): フォーマット済みログレイヤー
        // with_writer(): 出力先をfile_appenderに設定

        // --- 標準出力レイヤーを追加 ---
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stdout))
        // std::io::stdout: 標準出力（コンソール）

        // --- グローバル購読者として設定 ---
        .init();
        // init(): この購読者をグローバルに設定
        // 以降、info!, error!等のマクロがこの設定で動作
}

// ===================================================================
// 起動時処理関数
// ===================================================================
/// 起動時処理
///
/// WMI情報とネットワーク情報を取得し、設定ファイルを更新してサーバーに送信します。
///
/// # 引数
/// * `config` - クライアント設定への可変参照
/// * `config_path` - 設定ファイルパス
///
/// # 戻り値
/// * `Result<(), Box<dyn std::error::Error>>` - 成功時はOk(())、失敗時はErr
async fn initial_process(
    config: &mut ClientConfig,  // 可変参照: 関数内で変更可能
    config_path: &str            // &str: 文字列スライスへの参照（軽量）
) -> Result<(), Box<dyn std::error::Error>> {
    // async fn: 非同期関数
    // awaitポイントで他のタスクに制御を譲る

    info!("Running initial process");

    // --- WMI情報の収集 ---
    info!("Collecting WMI information");

    // WmiCollector::new(): 新しいWmiCollectorインスタンスを作成
    // ?演算子: Err時に関数から早期リターン
    let wmi_collector = WmiCollector::new()?;

    // collect_all(): すべてのWMI情報（UUID、モデル名、OS等）を収集
    // WmiInfo構造体が返る
    let wmi_data = wmi_collector.collect_all()?;

    // ログ出力: 収集した情報を記録
    info!("WMI information collected:");
    info!("  UUID: {}", wmi_data.uuid);          // .uuid: 構造体のフィールドアクセス
    info!("  Model: {}", wmi_data.model_name);
    info!("  OS: {} ({})", wmi_data.os, wmi_data.os_version);
    // 複数のプレースホルダー{}に順番に値が入る
    info!("  User: {}", wmi_data.user_name);

    // --- ネットワーク情報の検出 ---
    info!("Detecting network information");

    // NetworkDetector::get_active_adapter(): スタティックメソッド
    // アクティブなネットワークアダプタを検出
    let network_info = NetworkDetector::get_active_adapter()?;

    info!("Network information detected:");
    info!("  IP: {}", network_info.ip_address);
    info!("  MAC: {}", network_info.mac_address);
    info!("  Type: {}", network_info.network_type);

    // --- 設定ファイルの更新 ---
    // update_pc_info: ConfigのメソッドでPC情報を更新
    config.update_pc_info(
        // .clone(): String型をクローン（所有権を移さない）
        // Stringはヒープ上のデータなので、cloneでコピーが作られる
        wmi_data.uuid.clone(),
        network_info.mac_address.clone(),
        network_info.network_type.clone(),
        network_info.ip_address.clone(),
        wmi_data.os.clone(),
        wmi_data.os_version.clone(),
        wmi_data.model_name.clone(),
    );

    // --- 設定ファイルの保存 ---
    // save: TOMLファイルにシリアライズして書き込む
    config.save(config_path)?;
    info!("Configuration updated and saved");

    // --- サーバーへの送信 ---
    // send_to_server: 別の関数を呼び出し（await必要）
    send_to_server(config, config_path).await?;

    // 成功時
    Ok(())  // Ok(()): ユニット型()をOkでラップ
}

// ===================================================================
// 定期チェック処理関数
// ===================================================================
/// 定期チェック処理
///
/// ネットワーク情報を再取得し、サーバーに送信します。
/// WMI情報は変わらないので再取得しない。
async fn periodic_check(
    config: &mut ClientConfig,
    config_path: &str
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Running periodic check");

    // --- ネットワーク情報の再取得 ---
    // IPアドレスやMACアドレスが変わっている可能性がある
    let network_info = NetworkDetector::get_active_adapter()?;

    debug!("Network information updated:");
    debug!("  IP: {}", network_info.ip_address);
    debug!("  MAC: {}", network_info.mac_address);
    debug!("  Type: {}", network_info.network_type);

    // --- 設定ファイルのネットワーク情報を更新 ---
    // config.pc_info: PcInfoSettings構造体
    // = で直接代入（可変参照なので可能）
    config.pc_info.ip_address = network_info.ip_address;
    config.pc_info.mac_address = network_info.mac_address;
    config.pc_info.network_type = network_info.network_type;

    // --- サーバーに送信 ---
    send_to_server(config, config_path).await?;

    Ok(())
}

// ===================================================================
// サーバー送信関数
// ===================================================================
/// サーバーに送信
///
/// PC情報をJSON形式でサーバーのAPIエンドポイントにPOSTします。
async fn send_to_server(
    config: &mut ClientConfig,
    config_path: &str
) -> Result<(), Box<dyn std::error::Error>> {
    // --- PC情報の完全性チェック ---
    // UUID、OS、user_name等が空でないか確認
    if !config.is_pc_info_complete() {
        // !: 論理否定演算子
        warn!("PC information is incomplete, skipping send");
        return Ok(());  // エラーではなく正常終了扱い（スキップ）
    }

    // --- APIクライアントの作成 ---
    // ApiClient::new: コンストラクタ関数
    let api_client = ApiClient::new(
        config.server.url.clone(),              // サーバーURL
        config.server.request_timeout_secs,     // タイムアウト秒数
    )?;

    // --- 送信データの作成 ---
    // PcInfoData構造体を初期化
    // 構造体リテラル: フィールド名: 値の形式
    let data = PcInfoData {
        uuid: config.pc_info.uuid.clone(),
        mac_address: config.pc_info.mac_address.clone(),
        network_type: config.pc_info.network_type.clone(),
        user_name: config.pc_info.user_name.clone(),
        ip_address: config.pc_info.ip_address.clone(),
        os: config.pc_info.os.clone(),
        os_version: config.pc_info.os_version.clone(),
        model_name: config.pc_info.model_name.clone(),
    };

    // --- データ検証 ---
    // validate: データが有効かチェック（UUIDが空でない等）
    data.validate()?;

    // --- サーバーへの送信 ---
    info!("Sending PC information to server");

    // send_pc_info: HTTP POSTリクエストを送信（非同期）
    // &data: データへの参照を渡す
    let response = api_client.send_pc_info(&data).await?;
    // awaitポイント: ネットワークI/O完了まで待機

    info!("Server response: {} (action: {}, id: {})",
        response.status,   // "success"
        response.action,   // "created" or "updated"
        response.id);      // データベースのレコードID

    // --- 最終送信日時の更新 ---
    // Utc::now(): 現在のUTC時刻を取得
    // to_rfc3339(): RFC3339形式の文字列に変換
    //               例: "2025-10-25T12:34:56.789Z"
    let now = Utc::now().to_rfc3339();

    // update_last_send_datetime: 設定に送信日時を記録
    config.update_last_send_datetime(now);

    // save: 設定ファイルに保存
    config.save(config_path)?;

    info!("Last send datetime updated in config");

    Ok(())
}

// ===================================================================
// 送信判定関数
// ===================================================================
/// 送信が必要かチェック
///
/// 最終送信日時からsend_interval_secsが経過しているか確認します。
///
/// # 引数
/// * `config` - クライアント設定への参照
///
/// # 戻り値
/// * `bool` - true: 送信必要、false: 送信不要
async fn should_send(config: &ClientConfig) -> bool {
    // 戻り値型: bool（trueまたはfalse）
    // async fn: 非同期関数だが、awaitポイントがないので即座に完了

    // --- 最終送信日時が空の場合 ---
    if config.client.last_send_datetime.is_empty() {
        debug!("No last send datetime, should send");
        return true;  // 初回送信なので必要
    }

    // --- 最終送信日時のパース ---
    // parse_from_rfc3339: RFC3339形式の文字列をDateTime型にパース
    // Result<DateTime<FixedOffset>, ParseError>を返す
    match chrono::DateTime::parse_from_rfc3339(&config.client.last_send_datetime) {
        // パース成功時
        Ok(last_send) => {
            // last_send: DateTime<FixedOffset>型

            // 現在時刻を取得
            let now = Utc::now();

            // 経過時間を計算
            // with_timezone(&Utc): FixedOffsetからUtcに変換
            // signed_duration_since: 2つの時刻の差を計算
            let elapsed = now.signed_duration_since(last_send.with_timezone(&Utc));

            // 経過秒数を取得
            // num_seconds(): Duration型から秒数（i64）を取得
            let elapsed_secs = elapsed.num_seconds();

            debug!("Last send: {} ({} seconds ago)", last_send, elapsed_secs);

            // 経過秒数が送信間隔以上かチェック
            // as i64: u64をi64にキャスト（型変換）
            // >=: 以上演算子
            elapsed_secs >= config.client.send_interval_secs as i64
        }
        // パース失敗時
        Err(e) => {
            warn!("Failed to parse last send datetime: {}", e);
            // パースエラーの場合は安全のため送信する
            true
        }
    }
}

// ===================================================================
// リトライサイクル開始関数
// ===================================================================
/// リトライサイクルを開始
///
/// 既にリトライ中でない場合のみ、新しいリトライタスクをspawnします。
/// 二重起動を防ぐため、is_retyringフラグをチェックします。
///
/// # 引数
/// * `is_retrying` - リトライ中フラグ（Arc<Mutex<bool>>）
/// * `config_path` - 設定ファイルパス
async fn start_retry_cycle(
    is_retrying: Arc<Mutex<bool>>,  // Arc: 複数タスク間で共有
                                     // Mutex: 排他制御
    config_path: String              // String: 所有権を移す
) {
    // --- Mutexのロック取得 ---
    // .lock().await: Mutexをロック（非同期版）
    // 他のタスクがロック中ならここで待機
    let mut retrying = is_retrying.lock().await;
    // retrying: MutexGuard<bool>型（デリファレンスで*retryingでboolにアクセス）

    // --- 二重起動チェック ---
    if *retrying {
        // *retrying: MutexGuardをデリファレンス（中身のboolにアクセス）
        debug!("Already retrying, skipping new retry cycle");
        return;  // 既にリトライ中なので何もせず終了
    }
    // ここに到達したら、リトライ中ではない

    // --- リトライ開始フラグをセット ---
    *retrying = true;  // boolをtrueに設定

    // --- ロックの解放 ---
    drop(retrying);  // 明示的にMutexGuardをドロップ（ロック解放）
                     // スコープを抜けても自動的に解放されるが、
                     // 早めに解放することで他のタスクがロックを取れる

    info!("Starting retry cycle");

    // --- Arc のクローン ---
    // .clone(): Arcの参照カウントを増やす
    // 実際のMutex<bool>はコピーされない（共有される）
    let is_retrying_clone = is_retrying.clone();

    // --- 新しいタスクをspawn ---
    // tokio::spawn: 新しい非同期タスクを起動
    // async move { ... }: 非同期クロージャ
    // move: クロージャが所有権を奪う（外の変数をmove）
    tokio::spawn(async move {
        // この中で is_retrying_clone と config_path を使う
        // moveにより、これらの値の所有権がクロージャに移る

        // リトライサイクル処理を実行
        handle_retry_cycle(is_retrying_clone, config_path).await;
    });
    // spawnは即座に戻る（タスクはバックグラウンドで実行される）
}

// ===================================================================
// リトライサイクル処理関数
// ===================================================================
/// リトライサイクル処理
///
/// first_retry_delay_secs → second_retry_delay_secs を交互に繰り返します。
/// 送信に成功するまで無限にリトライします。
async fn handle_retry_cycle(
    is_retrying: Arc<Mutex<bool>>,
    config_path: String
) {
    // --- リトライ状態の初期化 ---
    let mut state = RetryState::FirstRetry;
    // 最初はFirstRetry状態

    // --- リトライループ ---
    loop {
        // --- 設定ファイルの読み込み ---
        // リトライ間隔設定を最新にするため、毎回読み込む
        let config = match ClientConfig::load(&config_path) {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to load config during retry: {}", e);
                // ロードエラー時は1分待機して次のループへ
                sleep(Duration::from_secs(60)).await;
                continue;  // ループの先頭に戻る
            }
        };

        // --- 待機時間の決定 ---
        // matchで状態に応じた待機時間を取得
        let delay_secs = match state {
            RetryState::FirstRetry => config.retry.first_retry_delay_secs,
            RetryState::SecondRetry => config.retry.second_retry_delay_secs,
        };

        info!("Retry scheduled in {} seconds (state: {:?})", delay_secs, state);

        // --- 待機 ---
        // sleep: 指定秒数待機（非同期スリープ）
        sleep(Duration::from_secs(delay_secs)).await;

        // --- リトライ送信 ---
        info!("Attempting retry send (state: {:?})", state);

        // retry_send: リトライ送信を試みる
        match retry_send(&config, &config_path).await {
            // 送信成功時
            Ok(_) => {
                info!("Retry send successful, exiting retry cycle");

                // リトライフラグをfalseに戻す
                let mut retrying = is_retrying.lock().await;
                *retrying = false;

                break;  // ループから抜ける（リトライサイクル終了）
            }
            // 送信失敗時
            Err(e) => {
                error!("Retry send failed: {}", e);

                // --- 次の状態に遷移 ---
                // matchで現在の状態に応じて次の状態を決定
                state = match state {
                    RetryState::FirstRetry => RetryState::SecondRetry,
                    // FirstRetry → SecondRetry
                    RetryState::SecondRetry => RetryState::FirstRetry,
                    // SecondRetry → FirstRetry（ループ）
                };
                // 次のloopで再度リトライ
            }
        }
    }
}

// ===================================================================
// リトライ送信関数
// ===================================================================
/// リトライ送信
///
/// ネットワーク情報を再取得してサーバーに送信します。
///
/// 戻り値型にSend + Syncトレイト境界が必要な理由:
/// tokio::spawnで使うエラー型は、スレッド間で安全に送信できる必要がある
async fn retry_send(
    _config: &ClientConfig,  // _config: 使わない引数（警告回避のため_を付ける）
    config_path: &str
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // + Send: スレッド間で送信可能
    // + Sync: 複数スレッドから安全にアクセス可能

    info!("Collecting information for retry send");

    // --- ネットワーク情報の再取得 ---
    let network_info = NetworkDetector::get_active_adapter()?;

    debug!("Network information updated:");
    debug!("  IP: {}", network_info.ip_address);
    debug!("  MAC: {}", network_info.mac_address);
    debug!("  Type: {}", network_info.network_type);

    // --- 設定の再読み込み ---
    // WMI情報などの最新データを取得
    let mut config = ClientConfig::load(config_path)?;

    // --- ネットワーク情報の更新 ---
    config.pc_info.ip_address = network_info.ip_address;
    config.pc_info.mac_address = network_info.mac_address;
    config.pc_info.network_type = network_info.network_type;

    // --- PC情報の完全性チェック ---
    if !config.is_pc_info_complete() {
        warn!("PC information is incomplete, cannot retry send");
        return Err("PC information is incomplete".into());
    }

    // --- APIクライアントの作成 ---
    let api_client = ApiClient::new(
        config.server.url.clone(),
        config.server.request_timeout_secs,
    )?;

    // --- 送信データの作成 ---
    let data = PcInfoData {
        uuid: config.pc_info.uuid.clone(),
        mac_address: config.pc_info.mac_address.clone(),
        network_type: config.pc_info.network_type.clone(),
        user_name: config.pc_info.user_name.clone(),
        ip_address: config.pc_info.ip_address.clone(),
        os: config.pc_info.os.clone(),
        os_version: config.pc_info.os_version.clone(),
        model_name: config.pc_info.model_name.clone(),
    };

    // --- データ検証 ---
    data.validate()?;

    // --- 送信 ---
    info!("Sending PC information to server (retry)");
    let response = api_client.send_pc_info(&data).await?;

    info!("Server response: {} (action: {}, id: {})",
        response.status, response.action, response.id);

    // --- 最終送信日時の更新 ---
    let now = Utc::now().to_rfc3339();
    config.update_last_send_datetime(now);
    config.save(config_path)?;

    info!("Last send datetime updated in config");

    Ok(())
}

// ===================================================================
// Rust学習のポイントまとめ
// ===================================================================
//
// 1. 所有権システム (Ownership)
//    - 各値には1つのオーナー（所有者）がいる
//    - オーナーがスコープを抜けると値がドロップされる
//    - 所有権を移す (move) か、借用 (borrow) するか選べる
//    - &T: 不変参照、&mut T: 可変参照
//
// 2. Result型とエラーハンドリング
//    - Result<T, E>: 成功時はOk(T)、失敗時はErr(E)
//    - ?演算子: Err時に早期リターン（関数から抜ける）
//    - match式: パターンマッチングで分岐
//
// 3. async/await
//    - async fn: 非同期関数（Futureを返す）
//    - .await: Futureの完了を待つ（ブロックしない）
//    - tokio: 非同期ランタイム（イベントループ実行）
//
// 4. Arc/Mutex
//    - Arc<T>: 複数スレッド間でTを共有（参照カウント）
//    - Mutex<T>: Tへの排他的アクセス（ロック）
//    - Arc<Mutex<T>>: スレッド間で共有 & 排他制御
//
// 5. Clone vs Copy
//    - Clone: 明示的なコピー（.clone()メソッド）
//    - Copy: 暗黙的なコピー（代入時に自動）
//    - String: Cloneのみ（ヒープデータ）
//    - i32, bool等: Copyも実装（スタックデータ）
//
// 6. トレイト (Trait)
//    - Rustのインターフェース
//    - Debug, Clone, Send, Sync等
//    - #[derive(...)]で自動実装
//
// 7. パターンマッチング
//    - match式: すべてのケースを網羅
//    - if let: 1つのパターンのみマッチ
//    - Some(x), Ok(x), Err(e)等のパターン
//
// ===================================================================
