use std::env::var;

use axum::{extract::State, response::IntoResponse as IntoRes, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use sqlx::{query, MySqlPool};

#[derive(Debug, Serialize, Deserialize)]
struct User {
    id: u64,
    name: String,
    password: String,
}

#[derive(Debug, Deserialize)]
struct Search {
    id: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct SqlResult<T> {
    message: String,
    data: Option<T>,
}

// ユーザーを登録する
async fn add_user(State(pool): State<MySqlPool>, Json(user): Json<User>) -> impl IntoRes {
    let result = query!(
        r#"
INSERT INTO user ( id, name, password )
VALUES ( ?, ?, ? );
    "#,
        user.id,
        user.name,
        user.password
    )
    .execute(&pool)
    .await;

    println!("{:#?}", result);

    match result {
        Ok(res) => {
            if res.rows_affected() <= 1 {
                return Json(SqlResult::<bool> {
                    message: "ユーザーの追加に成功しました".to_string(),
                    data: None,
                });
            }
        }
        Err(e) => eprintln!("{}", e),
    }

    Json(SqlResult {
        message: String::new(),
        data: None,
    })
}

// ユーザーを検索する
async fn search_user(State(pool): State<MySqlPool>, Json(search): Json<Search>) -> impl IntoRes {
    let result = query!(
        r#"
SELECT id, name, password
FROM user
WHERE id = ?;
"#,
        search.id
    )
    .fetch_all(&pool)
    .await;

    match result {
        Ok(user_list) => {
            let mut vec = Vec::new();
            for user in user_list {
                vec.push(User {
                    id: user.id as u64,
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
async fn delete_user(State(pool): State<MySqlPool>, Json(user): Json<User>) -> impl IntoRes {
    let result = query!(
        r#"
DELETE FROM user
WHERE id = ? AND name = ? AND password = ?
"#,
        user.id,
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

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let pool = MySqlPool::connect(&var("DATABASE_URL")?).await?;

    let app = Router::new()
        .route("/user/add", post(add_user))
        .route("/user/search", post(search_user))
        .route("/user/delete", post(delete_user))
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("192.168.11.6:9999").await?;
    axum::serve(listener, app).await?;

    Ok(())
}
