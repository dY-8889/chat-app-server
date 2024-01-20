use std::env::var;

use axum::{extract::State, response::IntoResponse as IntoRes, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use sqlx::{query, PgPool};

#[derive(Debug, Serialize, Deserialize)]
struct User {
    id: i32,
    name: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Search {
    id: u64,
    name: String,
}

#[derive(Debug, Serialize)]
struct SqlResult<T> {
    message: String,
    data: Option<T>,
}

#[derive(Debug, Serialize)]
struct ChatRoom {
    id: u16,
    name: String,
    password: String,
    message: Option<Vec<String>>,
    user_list: Vec<i16>,
}

#[derive(Debug, Deserialize)]
struct EnterRoom {
    room_id: i32,
    room_name: String,
    password: String,
    user_id: i32,
}

// ユーザーを登録する
async fn add_user(State(pool): State<PgPool>, Json(user): Json<User>) -> impl IntoRes {
    let result = query!(
        r#"
INSERT INTO users ( name, password )
VALUES ( $1, $2 );
    "#,
        user.name,
        user.password
    )
    .execute(&pool)
    .await;

    println!("{:#?}", result);

    match result {
        Ok(res) => {
            if res.rows_affected() <= 1 {
                return Json(SqlResult {
                    message: "ユーザーの追加に成功しました".to_string(),
                    data: Some(true),
                });
            }
        }
        Err(e) => eprintln!("{}", e),
    }

    Json(SqlResult {
        message: String::new(),
        data: Some(false),
    })
}

// ユーザーを検索する
async fn search_user(State(pool): State<PgPool>, Json(search): Json<Search>) -> impl IntoRes {
    let result = query!(
        r#"
SELECT id, name, password
FROM users
WHERE id = $1;
"#,
        search.id as i16
    )
    .fetch_all(&pool)
    .await;

    match result {
        Ok(user_list) => {
            let mut vec = Vec::new();
            for user in user_list {
                vec.push(User {
                    id: user.id,
                    name: user.name,
                    password: user.password,
                });
            }
            return Json(SqlResult {
                message: "ユーザーが見つかりました".to_string(),
                data: Some(vec),
            });
        }
        Err(e) => eprintln!("{}", e),
    }

    Json(SqlResult {
        message: "条件に該当するユーザーが見つかりませんでした".to_string(),
        data: Some(Vec::new()),
    })
}

// ユーザーを削除する
async fn delete_user(State(pool): State<PgPool>, Json(user): Json<User>) -> impl IntoRes {
    let result = query!(
        r#"
DELETE FROM users
WHERE id = $1 AND name = $2 AND password = $3;
"#,
        user.id as i16,
        user.name,
        user.password
    )
    .execute(&pool)
    .await;

    println!("{:#?}", result);

    match result {
        Ok(res) => {
            if 0 < res.rows_affected() {
                return Json(SqlResult {
                    message: "削除に成功しました!".to_string(),
                    data: Some(res.rows_affected()),
                });
            }
        }
        Err(e) => eprintln!("{}", e),
    }

    Json(SqlResult {
        message: "削除するUserが見つかりませんでした".to_string(),
        data: Some(0),
    })
}

// 新しく部屋を作る
async fn create_room(State(pool): State<PgPool>, Json(room): Json<EnterRoom>) -> impl IntoRes {
    println!("{:#?}", room);
    let result = query!(
        r#"
INSERT INTO chat_room ( name, password )
VALUES ( $1, $2 );
    "#,
        room.room_name,
        room.password
    )
    .execute(&pool)
    .await;

    println!("{:#?}", result);

    match result {
        Ok(_) => {
            return Json(SqlResult {
                message: "部屋の作成に成功しました".to_string(),
                data: Some(true),
            });
        }

        Err(e) => eprintln!("{}", e),
    }

    Json(SqlResult {
        message: "部屋の作成に失敗しました".to_string(),
        data: Some(false),
    })
}

// 部屋に入る
async fn enter_room(State(pool): State<PgPool>, Json(room): Json<EnterRoom>) -> impl IntoRes {
    println!("{:#?}", room);
    // chat_roomテーブルのuser_listにidを追加する
    let result = query!(
        r#"
UPDATE chat_room
SET user_list = array_append(user_list, $1)
WHERE id = $2 AND name = $3 AND password = $4;
    "#,
        room.user_id,
        room.room_id,
        room.room_name,
        room.password
    )
    .execute(&pool)
    .await;

    println!("enter room {:#?}", result);

    match result {
        Ok(_) => {
            return Json(SqlResult {
                message: "入室に成功しました".to_string(),
                data: Some(true),
            });
        }

        Err(e) => eprintln!("{}", e),
    }

    Json(SqlResult {
        message: "入室に失敗".to_string(),
        data: None,
    })
}

#[derive(Debug, Deserialize)]
struct Message {
    text: String,
    room_id: i32,
}

//
async fn message_get(State(pool): State<PgPool>, Json(room_id): Json<i32>) -> impl IntoRes {
    println!("get to {}", room_id);
    let result = query!(
        r#"
SELECT message
FROM chat_room
WHERE id = $1
    "#,
        room_id
    )
    .fetch_one(&pool)
    .await;

    match result {
        Ok(data) => {
            let data = data.message.unwrap();
            return Json(SqlResult {
                message: "メッセージの読み取りに成功".to_string(),
                data: Some(data),
            });
        }
        Err(e) => eprintln!("{}", e),
    }

    Json(SqlResult {
        message: "メッセージの読み取りに成功".to_string(),
        data: None,
    })
}

// 送られてきたメッセージをDBに保存する
async fn message_send(State(pool): State<PgPool>, Json(message): Json<Message>) -> impl IntoRes {
    let result = query!(
        r#"
UPDATE chat_room
SET message = array_append(message, $1)
WHERE id = $2;
    "#,
        message.text,
        message.room_id
    )
    .execute(&pool)
    .await;

    match result {
        Ok(_) => {
            return Json(SqlResult {
                message: "メッセージの送信に成功".to_string(),
                data: Some(true),
            });
        }
        Err(e) => eprintln!("{}", e),
    }

    Json(SqlResult {
        message: "メッセージの送信に成功".to_string(),
        data: None,
    })
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let pool = PgPool::connect(&var("DATABASE_URL").expect("環境変数が見つかりません")).await?;

    let app = Router::new()
        .route("/user/add", post(add_user))
        .route("/user/search", post(search_user))
        .route("/user/delete", post(delete_user))
        .route("/room/create", post(create_room))
        .route("/room/enter", post(enter_room))
        .route("/message/get", post(message_get))
        .route("/message/send", post(message_send))
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:9999").await?;
    axum::serve(listener, app).await?;

    Ok(())
}
