use axum::{extract::{Path, State}, http::StatusCode, response::IntoResponse, Json};
use futures::TryStreamExt;
use mongodb::{bson::{doc, oid::ObjectId, to_document}, results::{DeleteResult, InsertOneResult, UpdateResult}, Collection};
use serde_json::Value;

pub async fn get_all(Path(collection): Path<String>,State(db): State<mongodb::Database>) -> Result<Json<Vec<Value>>, (axum::http::StatusCode, String)>  {
    let coll: Collection<Value> = db.collection(&collection);

    let mut cursor = coll.find(doc! {}).await.map_err(|e| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("DB error: {}", e),
        )
    })?;

    let mut results = Vec::new();
    while let Some(doc) = cursor.try_next().await.map_err(|e| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Cursor error: {}", e),
        )
    })? {
        results.push(doc);
    }

    Ok(Json(results))
}

pub async fn get_one(Path((collection, id)): Path<(String, String)>, State(db): State<mongodb::Database>) -> impl IntoResponse {
    let coll: Collection<Value> = db.collection(&collection);
    let obj_id = match ObjectId::parse_str(&id) {
        Ok(id) => id,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid ObjectId").into_response(),
    };

    match coll.find_one(doc! { "_id": obj_id }).await {
        Ok(Some(doc)) => (StatusCode::OK, Json(doc)).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "Not Found").into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "DB Error").into_response(),
    }
}

pub async fn create(
    Path(collection): Path<String>,
    State(db): State<mongodb::Database>,
    Json(input): Json<Value>) -> Result<Json<InsertOneResult>, (StatusCode, String)> {
    let result = db.collection(&collection).insert_one(input).await.map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("insert failed: {}", e))
    })?;
    Ok(Json(result))
}

pub async fn update(
    Path((collection, id)): Path<(String, String)>,
    State(db): State<mongodb::Database>,
    Json(input): Json<Value>) -> Result<Json<UpdateResult>, (StatusCode, String)> {

    let obj_id = match ObjectId::parse_str(&id) {
        Ok(id) => id,
        Err(_) => return Err(
            (StatusCode::BAD_REQUEST, "Invalid ObjectId".to_string())),
    };
    let coll: Collection<Value> = db.collection(&collection);
    let filter = doc! { "_id": obj_id };
    let update = doc! { "$set": to_document(&input).unwrap() };
    let res = coll.update_one(filter, update).await.unwrap();

    Ok(Json(res))
}

pub async fn delete(
    Path((collection, id)): Path<(String, String)>,
    State(db): State<mongodb::Database>,
) -> impl IntoResponse {
    let obj_id = match ObjectId::parse_str(&id) {
        Ok(id) => id,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid ObjectId").into_response(),
    };

    let coll: Collection<Value> = db.collection(&collection);
    let filter = doc! { "_id": obj_id };

    let result: DeleteResult = match coll.delete_one(filter).await {
        Ok(r) => r,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "DB Error").into_response(),
    };

    if result.deleted_count == 1 {
        StatusCode::NO_CONTENT.into_response() // ✅ 204
    } else {
        (StatusCode::NOT_FOUND, "Not Found").into_response()
    }
}
