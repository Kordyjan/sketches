use crate::model::{Chapter, ChapterDetail, Snapshot};
use crate::state::ChaptersState;
use rocket::serde::json::Json;
use rocket::{get, launch, routes, State};
use rocket_cors::{AllowedOrigins, CorsOptions};

mod model;
mod state;

#[launch]
fn rocket() -> _ {
    let config = CorsOptions::default()
        .allowed_origins(AllowedOrigins::All)
        .to_cors()
        .unwrap();

    rocket::build()
        .manage(ChaptersState::read("tmp/trace.json"))
        .attach(config)
        .mount("/", routes![chapter_list, chapter_detail, snapshot])
}

#[get("/chapters")]
fn chapter_list(state: &State<ChaptersState>) -> Json<Vec<Chapter>> {
    Json(state.get_chapters())
}

#[get("/chapters/<ch>")]
fn chapter_detail(state: &State<ChaptersState>, ch: usize) -> Option<Json<ChapterDetail>> {
    state.get_chapter_detail(ch).map(Json)
}

#[get("/chapters/<ch>/<op>/snapshot")]
fn snapshot(state: &State<ChaptersState>, ch: usize, op: usize) -> Option<Json<Snapshot>> {
    state.get_snapshot(ch, op).map(Json)
}
