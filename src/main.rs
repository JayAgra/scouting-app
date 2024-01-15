use std::{env, io, collections::HashMap, pin::Pin, sync::RwLock};
use actix_governor::{Governor, GovernorConfigBuilder};
use actix_http::StatusCode;
use actix_identity::{CookieIdentityPolicy, Identity, IdentityService};
use actix_session::{SessionMiddleware, Session};
use actix_web::{error, middleware::{self, DefaultHeaders}, web, App, Error as AWError, HttpRequest, HttpResponse, HttpServer, cookie::Key, Responder, FromRequest, dev::Payload};
use actix_web_static_files::ResourceFiles;
use dotenv::dotenv;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use r2d2_sqlite::{self, SqliteConnectionManager};
use serde::{Serialize, Deserialize};
use webauthn_rs::prelude::*;

mod analyze;
mod auth;
mod casino;
mod db_auth;
mod db_main;
mod db_transact;
mod forward;
mod passkey;
mod session;
mod static_files;

// hashmap containing user session IDs
#[derive(Serialize, Deserialize, Default, Clone)]
struct Sessions {
    user_map: HashMap<String, db_auth::User>
}

// gets a user object from requests. needed for db_auth::User param in handlers
impl FromRequest for db_auth::User {
    type Error = actix_web::Error;
    type Future = Pin<Box<dyn futures_util::Future<Output = Result<db_auth::User, Self::Error>>>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        let fut = Identity::from_request(req, payload);
        let session: Option<&web::Data<RwLock<Sessions>>> = req.app_data();
        if session.is_none() {
            return Box::pin(
                async {
                    Err(error::ErrorUnauthorized("{\"status\": \"unauthorized\"}"))
                }
            );
        }
        let session = session.unwrap().clone();
        Box::pin(async move {
            if let Some(identity) = fut.await?.identity() {
                if let Some(user) = session.read().unwrap().user_map.get(&identity).map(|x| x.clone()) {
                    return Ok(user);
                }
            };
            Err(error::ErrorUnauthorized("{\"status\": \"unauthorized\"}"))
        })
    }
}

// allows three separate pools in the single web::Data<Databases> parameter
struct Databases {
    main: db_main::Pool,
    auth: db_auth::Pool,
    transact: db_transact::Pool
}

// create secret key. probably could/should be an environment variable
fn get_secret_key() -> Key {
    Key::generate()
}

// send a 401. used in many management endpoints
fn unauthorized_response() -> HttpResponse {
    HttpResponse::Unauthorized()
        .status(StatusCode::from_u16(401).unwrap())
        .insert_header(("Cache-Control", "no-cache"))
        .body("{\"status\": \"unauthorized\"}")
}

// create account endpoint
async fn auth_post_create(db: web::Data<Databases>, data: web::Json<auth::CreateForm>) -> impl Responder {
    auth::create_account(&db.auth, data).await
}

// login endpoint
async fn auth_post_login(db: web::Data<Databases>, session: web::Data<RwLock<Sessions>>, identity: Identity, data: web::Json<auth::LoginForm>) -> impl Responder {
    auth::login(&db.auth, session, identity, data).await
}

// destroy session endpoint
async fn auth_get_logout(session: web::Data<RwLock<Sessions>>, identity: Identity) -> impl Responder {
    auth::logout(session, identity).await
}

// create passkey
async fn auth_psk_create_start(db: web::Data<Databases>, user: db_auth::User, session: Session, webauthn: web::Data<Webauthn>) -> Result<HttpResponse, AWError> {
    Ok(
        HttpResponse::Ok()
            .insert_header(("Cache-Control", "no-cache"))
            .json(passkey::webauthn_start_registration(&db.auth, user, session, webauthn).await?)
    )
}

// finish passkey creation
async fn auth_psk_create_finish(db: web::Data<Databases>, user: db_auth::User, data: web::Json<RegisterPublicKeyCredential>, session: Session, webauthn: web::Data<Webauthn>) -> Result<HttpResponse, AWError> {
    Ok(
        passkey::webauthn_finish_registration(&db.auth, user, data, session, webauthn).await?
    )
}

// get passkey auth challenge
async fn auth_psk_auth_start(db: web::Data<Databases>, username: web::Path<String>, session: Session, webauthn: web::Data<Webauthn>) -> Result<HttpResponse, AWError> {
    Ok(
        HttpResponse::Ok()
            .insert_header(("Cache-Control", "no-cache"))
            .json(passkey::webauthn_start_authentication(&db.auth, username.into_inner(), session, webauthn).await?)
    )
}

// finish passkey authentication
async fn auth_psk_auth_finish(db: web::Data<Databases>, cred: web::Json<PublicKeyCredential>, session: Session, identity: Identity, webauthn: web::Data<Webauthn>, sessions: web::Data<RwLock<Sessions>>) -> Result<HttpResponse, AWError> {
    Ok(
        passkey::webauthn_finish_authentication(&db.auth, cred, session, identity, webauthn, sessions).await?
    )
}

// get detailed data by submission id. used in /detail
async fn data_get_detailed(path: web::Path<String>, db: web::Data<Databases>, _user: db_auth::User) -> Result<HttpResponse, AWError> {
    Ok(
        HttpResponse::Ok()
            .insert_header(("Cache-Control", "no-cache"))
            .json(db_main::execute(&db.main, db_main::MainData::GetDataDetailed, path).await?)
    )
}

// check if a submission exists, by id. used in submit script to verify submission (verification is mostly a gimmick but whatever)
async fn data_get_exists(path: web::Path<String>, db: web::Data<Databases>, _user: db_auth::User) -> Result<HttpResponse, AWError> {
    Ok(
        HttpResponse::Ok()
            .insert_header(("Cache-Control", "no-cache"))
            .json(db_main::execute(&db.main, db_main::MainData::DataExists, path).await?)
    )
}

// get summary of all data for a given team at an event in a season. used on /browse
async fn data_get_main_brief_team(path: web::Path<String>, db: web::Data<Databases>, _user: db_auth::User) -> Result<HttpResponse, AWError> {
    Ok(
        HttpResponse::Ok()
            .insert_header(("Cache-Control", "no-cache"))
            .json(db_main::execute(&db.main, db_main::MainData::BriefTeam, path).await?)
    )
}

// get summary of all data for a given match at an event, in a specified season. used on /browsw
async fn data_get_main_brief_match(path: web::Path<String>, db: web::Data<Databases>, _user: db_auth::User) -> Result<HttpResponse, AWError> {
    Ok(
        HttpResponse::Ok()
            .insert_header(("Cache-Control", "no-cache"))
            .json(db_main::execute(&db.main, db_main::MainData::BriefMatch, path).await?)
    )
}

// get summary of all data from an event, given a season. used for /browse
async fn data_get_main_brief_event(path: web::Path<String>, db: web::Data<Databases>, _user: db_auth::User) -> Result<HttpResponse, AWError> {
    Ok(
        HttpResponse::Ok()
            .insert_header(("Cache-Control", "no-cache"))
            .json(db_main::execute(&db.main, db_main::MainData::BriefEvent, path).await?)
    )
}

// get summary of all submissions created by a certain user id. used for /browse
async fn data_get_main_brief_user(path: web::Path<String>, db: web::Data<Databases>, _user: db_auth::User) -> Result<HttpResponse, AWError> {
    Ok(
        HttpResponse::Ok()
            .insert_header(("Cache-Control", "no-cache"))
            .json(db_main::execute(&db.main, db_main::MainData::BriefUser, path).await?)
    )
}

// get basic data about all teams at an event, in a season. used for event rankings. ** NO AUTH **
async fn data_get_main_teams(path: web::Path<String>, db: web::Data<Databases>) -> Result<HttpResponse, AWError> {
    Ok(
        HttpResponse::Ok()
            .insert_header(("Cache-Control", "no-cache"))
            .json(db_main::execute(&db.main, db_main::MainData::GetTeams, path).await?)
    )
}

// get POSTed data from form
async fn data_post_submit(data: web::Json<db_main::MainInsert>, db: web::Data<Databases>, user: db_auth::User) -> Result<HttpResponse, AWError> {
    Ok(
        HttpResponse::Ok()
            .insert_header(("Cache-Control", "no-cache"))
            .json(db_main::execute_insert(&db.main, &db.transact, &db.auth, data, user).await?)
    )
}

// forward frc api data for teams [deprecated]
async fn event_get_frc_api(req: HttpRequest, path: web::Path<(String, String)>, _user: db_auth::User) -> HttpResponse {
    forward::forward_frc_api_event_teams(req, path).await
}

// forward frc api data for events. used on main form to ensure entered matches and teams are valid
async fn event_get_frc_api_matches(req: HttpRequest, path: web::Path<(String, String, String, String)>, _user: db_auth::User) -> HttpResponse {
    forward::forward_frc_api_event_matches(req, path).await
}

// get all valid submission IDs. used on /manage to create list of IDs that can be acted on
async fn manage_get_submission_ids(path: web::Path<String>, db: web::Data<Databases>, user: db_auth::User) -> Result<HttpResponse, AWError> {
    if user.admin == "true" {
        Ok(
            HttpResponse::Ok()
                .insert_header(("Cache-Control", "no-cache"))
                .json(db_main::execute(&db.main, db_main::MainData::Id, path).await?)
        )
    } else {
        Ok(unauthorized_response())
    }
}

// gets list of all valid user ids, used in /manageScouts
async fn manage_get_all_users(db: web::Data<Databases>, user: db_auth::User) -> Result<HttpResponse, AWError> {
    if user.admin == "true" {
        Ok(
            HttpResponse::Ok()
                .insert_header(("Cache-Control", "no-cache"))
                .json(db_auth::execute_get_users_mgmt(&db.auth, db_auth::UserQueryType::All, user).await?)
        )
    } else {
        Ok(unauthorized_response())
    }
}

// gets list of users in a team, used in /manageTeam
async fn manage_get_all_users_in_team(db: web::Data<Databases>, user: db_auth::User) -> Result<HttpResponse, AWError> {
    if user.admin == "true" || user.team_admin != 0 {
        Ok(
            HttpResponse::Ok()
                .insert_header(("Cache-Control", "no-cache"))
                .json(db_auth::execute_get_users_mgmt(&db.auth, db_auth::UserQueryType::Team, user).await?)
        )
    } else {
        Ok(unauthorized_response())
    }
}

// gets all access keys, used for /manageTeams
async fn manage_get_all_keys(db: web::Data<Databases>, user: db_auth::User) -> Result<HttpResponse, AWError> {
    if user.admin == "true" {
        Ok(
            HttpResponse::Ok()
                .insert_header(("Cache-Control", "no-cache"))
                .json(db_auth::get_access_key(&db.auth, "".to_string(), db_auth::AccessKeyQuery::AllKeys).await?)
        )
    } else {
        Ok(unauthorized_response())
    }
}

// DELETE endpoint to remove a submission. used in /manage
async fn manage_delete_submission(db: web::Data<Databases>, user: db_auth::User, path: web::Path<String>) -> Result<HttpResponse, AWError> {
    if user.admin == "true" {
        Ok(
            HttpResponse::Ok()
                .insert_header(("Cache-Control", "no-cache"))
                .body(db_main::delete_by_id(&db.main, &db.transact, &db.auth, path).await?)
        )
    } else {
        Ok(unauthorized_response())
    }
}

// DELETE endpoint to remove a user, used in /manageScouts
async fn manage_delete_user(req: HttpRequest, db: web::Data<Databases>, user: db_auth::User) -> Result<HttpResponse, AWError> {
    if user.admin == "true" {
        Ok(
            HttpResponse::Ok()
                .insert_header(("Cache-Control", "no-cache"))
                .body(db_auth::execute_manage_user(&db.auth, db_auth::UserManageAction::DeleteUser, [req.match_info().get("user_id").unwrap().parse().unwrap(), "".to_string()]).await?)
        )
    } else {
        Ok(unauthorized_response())
    }
}

// DELETE endpoint to remove a user, but for a team admin (requires that target user is member of team). used in /manageTeam
async fn manage_delete_user_team_admin(req: HttpRequest, db: web::Data<Databases>, user: db_auth::User) -> Result<HttpResponse, AWError> {
    if user.admin == "true" || user.team_admin != 0 {
        if user.admin == "true" || db_auth::get_user_id(&db.auth, req.match_info().get("user_id").unwrap().parse().unwrap()).await?.team == user.team_admin {
            Ok(
                HttpResponse::Ok()
                    .insert_header(("Cache-Control", "no-cache"))
                    .body(db_auth::execute_manage_user(&db.auth, db_auth::UserManageAction::DeleteUser, [req.match_info().get("user_id").unwrap().parse().unwrap(), "".to_string()]).await?)
            )
        } else {
            Ok(unauthorized_response())
        }
    } else {
        Ok(unauthorized_response())
    }
}

// DELETE endpoint to 86 an access key, used in /manageTeams
async fn manage_delete_access_key(req: HttpRequest, db: web::Data<Databases>, user: db_auth::User) -> Result<HttpResponse, AWError> {
    if user.admin == "true" {
        Ok(
            HttpResponse::Ok()
                .insert_header(("Cache-Control", "no-cache"))
                .body(db_auth::delete_access_key(&db.auth, req.match_info().get("access_key_id").unwrap().parse().unwrap()).await?)
        )
    } else {
        Ok(unauthorized_response())
    }
}

// patch to update a user's administration status, used in /manageScouts
async fn manage_patch_admin(req: HttpRequest, db: web::Data<Databases>, user: db_auth::User) -> Result<HttpResponse, AWError> {
    if user.admin == "true" {
        Ok(
            HttpResponse::Ok()
                .insert_header(("Cache-Control", "no-cache"))
                .body(db_auth::execute_manage_user(&db.auth, db_auth::UserManageAction::ModifyAdmin, [req.match_info().get("admin").unwrap().parse().unwrap(), req.match_info().get("user_id").unwrap().parse().unwrap()]).await?)
        )
    } else {
        Ok(unauthorized_response())
    }
}

// patch to update a user's [team] administration status, used in /manageScouts
async fn manage_patch_team_admin(req: HttpRequest, db: web::Data<Databases>, user: db_auth::User) -> Result<HttpResponse, AWError> {
    if user.admin == "true" {
        Ok(
            HttpResponse::Ok()
                .insert_header(("Cache-Control", "no-cache"))
                .body(db_auth::execute_manage_user(&db.auth, db_auth::UserManageAction::ModifyTeamAdmin, [req.match_info().get("admin").unwrap().parse().unwrap(), req.match_info().get("user_id").unwrap().parse().unwrap()]).await?)
        )
    } else {
        Ok(unauthorized_response())
    }
}

// patch to update a user's points, used in /manageScouts
async fn manage_patch_points(req: HttpRequest, db: web::Data<Databases>, user: db_auth::User) -> Result<HttpResponse, AWError> {
    if user.admin == "true" {
        Ok(
            HttpResponse::Ok()
                .insert_header(("Cache-Control", "no-cache"))
                .body(db_auth::execute_manage_user(&db.auth, db_auth::UserManageAction::ModifyPoints, [req.match_info().get("modify").unwrap().parse().unwrap(), req.match_info().get("user_id").unwrap().parse().unwrap()]).await?)
        )
    } else {
        Ok(unauthorized_response())
    }
}

// patch to modify an existing access key, used in /manageTeams
async fn manage_patch_access_key(req: HttpRequest, db: web::Data<Databases>, user: db_auth::User) -> Result<HttpResponse, AWError> {
    if user.admin == "true" {
        Ok(
            HttpResponse::Ok()
                .insert_header(("Cache-Control", "no-cache"))
                .body(db_auth::update_access_key(&db.auth, req.match_info().get("key").unwrap().parse().unwrap(), req.match_info().get("id").unwrap().parse().unwrap()).await?)
        )
    } else {
        Ok(unauthorized_response())
    }
}

// post to create a new access key, used in /manageTeams
async fn manage_post_access_key(req: HttpRequest, db: web::Data<Databases>, user: db_auth::User) -> Result<HttpResponse, AWError> {
    if user.admin == "true" {
        Ok(
            HttpResponse::Ok()
                .insert_header(("Cache-Control", "no-cache"))
                .body(db_auth::create_access_key(&db.auth, req.match_info().get("key").unwrap().parse().unwrap(), req.match_info().get("team").unwrap().parse().unwrap()).await?)
        )
    } else {
        Ok(unauthorized_response())
    }
}

// get transactions, used in /pointRecords
async fn misc_get_transact_me(db: web::Data<Databases>, user: db_auth::User) -> Result<HttpResponse, AWError> {
    Ok(
        HttpResponse::Ok()
            .insert_header(("Cache-Control", "no-cache"))
            .json(db_transact::execute(&db.transact, db_transact::TransactData::GetUserTransactions, user).await?)
        )
}

// get to confirm session status and obtain current user id. used in main form to ensure session is active
async fn misc_get_whoami(user: db_auth::User) -> Result<HttpResponse, AWError> {
    Ok(
        HttpResponse::Ok()
            .insert_header(("Cache-Control", "no-cache"))
            .json(db_main::Id { id: user.id })
    )
}

// get all points. used to construct the leaderboard
async fn points_get_all(db: web::Data<Databases>, _user: db_auth::User) -> Result<HttpResponse, AWError> {
    Ok(
        HttpResponse::Ok()
            .insert_header(("Cache-Control", "no-cache"))
            .json(db_auth::execute_scores(&db.auth, db_auth::AuthData::GetUserScores).await?)
    )
}

// get spin wheel for the casino
async fn casino_wheel(db: web::Data<Databases>, user: db_auth::User) -> Result<HttpResponse, AWError> {
    Ok(
        HttpResponse::Ok()
            .insert_header(("Cache-Control", "no-cache"))
            .body(casino::spin_thing(&db.auth, &db.transact, user).await?)
    )
}

// get for debugging. returns the current user object.
async fn debug_get_user(user: db_auth::User) -> Result<HttpResponse, AWError> {
    Ok(
        HttpResponse::Ok()
            .insert_header(("Cache-Control", "no-cache"))
            .json(user)
    )
}

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

#[actix_web::main]
async fn main() -> io::Result<()> {
    // load environment variables from .env file
    dotenv().ok();

    // don't log all that shit when in release mode
    if cfg!(debug_assertions) {
        env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    } else {
        env_logger::init_from_env(env_logger::Env::new().default_filter_or("error"));
        println!("[OK] starting in release mode");
    }

    // hashmap w: web::Data<RwLock<Sessions>>ith user sessions in it
    let sessions: web::Data<RwLock<Sessions>> = web::Data::new(RwLock::new(Sessions {
        user_map: HashMap::new(),
    }));

    // main database connection
    let main_db_manager = SqliteConnectionManager::file("data.db");
    let main_db_pool = db_main::Pool::new(main_db_manager).unwrap();
    let main_db_connection = main_db_pool.get().expect("main db: connection failed");
    main_db_connection.execute_batch("PRAGMA journal_mode=WAL;").expect("main db: WAL failed");
    drop(main_db_connection);

    // auth database connection
    let auth_db_manager = SqliteConnectionManager::file("data_auth.db");
    let auth_db_pool = db_main::Pool::new(auth_db_manager).unwrap();
    let auth_db_connection = auth_db_pool.get().expect("auth db: connection failed");
    auth_db_connection.execute_batch("PRAGMA journal_mode=WAL;").expect("auth db: WAL failed");
    drop(auth_db_connection);

    // transaction database connection
    let trans_db_manager = SqliteConnectionManager::file("data_transact.db");
    let trans_db_pool = db_main::Pool::new(trans_db_manager).unwrap();
    let trans_db_connection = trans_db_pool.get().expect("trans db: connection failed");
    trans_db_connection.execute_batch("PRAGMA journal_mode=WAL;").expect("trans db: WAL failed");
    drop(trans_db_connection);

    // create secret key for uh cookies i think
    let secret_key = get_secret_key();

    // ratelimiting with governor
    let governor_conf = GovernorConfigBuilder::default()
        // these may be a lil high but whatever
        .per_second(100)
        .burst_size(200)
        .finish()
        .unwrap();

    /*
     *  generate a self-signed certificate for localhost (run from bearTracks directory):
     *  openssl req -x509 -newkey rsa:4096 -nodes -keyout ./ssl/key.pem -out ./ssl/cert.pem -days 365 -subj '/CN=localhost'
     */
    // create ssl builder for tls config
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder.set_private_key_file("./ssl/key.pem", SslFiletype::PEM).unwrap();
    builder.set_certificate_chain_file("./ssl/cert.pem").unwrap();

    // config done. now, create the new HttpServer
    log::info!("[OK] starting bearTracks on port 443 and 80");

    HttpServer::new(move || {
        // generated resources from actix_web_files
        let generated = generate();
        App::new()
            // add databases to app data
            .app_data(web::Data::new(Databases {main: main_db_pool.clone(), auth: auth_db_pool.clone(), transact: trans_db_pool.clone() }))
            // add sessions to app data
            .app_data(sessions.clone())
            // add webauthn to app data
            .app_data(passkey::setup_passkeys())
            // use governor ratelimiting as middleware
            .wrap(Governor::new(&governor_conf))
            // use cookie id system middleware
            .wrap(IdentityService::new(
                CookieIdentityPolicy::new(&[0; 32])
                    .name("bear_tracks")
                    .secure(false),
            ))
            // logging middleware
            .wrap(middleware::Logger::default())
            // session middleware
            .wrap(
                SessionMiddleware::builder(session::MemorySession, secret_key.clone())
                    .cookie_name("bear_tracks-ms".to_string())
                    .cookie_http_only(true)
                    .cookie_secure(false)
                    .build()
            )
            // default headers for caching. overridden on most all api endpoints
            .wrap(DefaultHeaders::new().add(("Cache-Control", "public, max-age=23328000")).add(("X-bearTracks", "4.0.0")))
            /* src  endpoints */
                // GET individual files
                .route("/", web::get().to(static_files::static_index))
                .route("/blackjack", web::get().to(static_files::static_blackjack))
                .route("/charts", web::get().to(static_files::static_charts))
                .route("/create", web::get().to(static_files::static_create))
                .route("/login", web::get().to(static_files::static_login))
                .route("/manage", web::get().to(static_files::static_manage))
                .route("/manageScouts", web::get().to(static_files::static_manage_scouts))
                .route("/manageTeam", web::get().to(static_files::static_manage_team))
                .route("/manageTeams", web::get().to(static_files::static_manage_teams))
                .route("/passkey", web::get().to(static_files::static_passkey))
                .route("/pointRecords", web::get().to(static_files::static_point_records))
                .route("/points", web::get().to(static_files::static_points))
                .route("/scouts", web::get().to(static_files::static_scouts))
                .route("/settings", web::get().to(static_files::static_settings))
                .route("/spin", web::get().to(static_files::static_spin))
                .route("/teams", web::get().to(static_files::static_teams))
                .route("/favicon.ico", web::get().to(static_files::static_favicon))
                // GET folders
                .service(ResourceFiles::new("/static", generated))
            /* auth endpoints */
                // GET
                .service(web::resource("/logout").route(web::get().to(auth_get_logout)))
                // POST
                .service(web::resource("/api/v1/auth/create").route(web::post().to(auth_post_create)))
                .service(web::resource("/api/v1/auth/login").route(web::post().to(auth_post_login)))
                .service(web::resource("/api/v1/auth/passkey/register_start").route(web::post().to(auth_psk_create_start)))
                .service(web::resource("/api/v1/auth/passkey/register_finish").route(web::post().to(auth_psk_create_finish)))
                .service(web::resource("/api/v1/auth/passkey/auth_start/{username}").route(web::post().to(auth_psk_auth_start)))
                .service(web::resource("/api/v1/auth/passkey/auth_finish").route(web::post().to(auth_psk_auth_finish)))
            /* data endpoints */
                // GET (✅)
                .service(web::resource("/api/v1/data/detail/{id}").route(web::get().to(data_get_detailed)))
                .service(web::resource("/api/v1/data/exists/{id}").route(web::get().to(data_get_exists)))
                .service(web::resource("/api/v1/data/brief/team/{args}*").route(web::get().to(data_get_main_brief_team))) // season/event/team
                .service(web::resource("/api/v1/data/brief/match/{args}*").route(web::get().to(data_get_main_brief_match))) // season/event/match_num
                .service(web::resource("/api/v1/data/brief/event/{args}*").route(web::get().to(data_get_main_brief_event))) // season/event
                .service(web::resource("/api/v1/data/brief/user/{args}*").route(web::get().to(data_get_main_brief_user))) // season/user_id
                .service(web::resource("/api/v1/data/teams/{args}*").route(web::get().to(data_get_main_teams))) // season/event
                .service(web::resource("/api/v1/events/teams/{season}/{event}").route(web::get().to(event_get_frc_api)))
                .service(web::resource("/api/v1/events/matches/{season}/{event}/{level}/{all}").route(web::get().to(event_get_frc_api_matches)))
                // POST (✅)
                .service(web::resource("/api/v1/data/submit").route(web::post().to(data_post_submit)))
            /* manage endpoints */
                // GET
                .service(web::resource("/api/v1/manage/submission_ids/{args}").route(web::get().to(manage_get_submission_ids)))
                .service(web::resource("/api/v1/manage/all_users").route(web::get().to(manage_get_all_users)))
                .service(web::resource("/api/v1/manage/team_users").route(web::get().to(manage_get_all_users_in_team)))
                .service(web::resource("/api/v1/manage/all_access_keys").route(web::get().to(manage_get_all_keys)))
                // DELETE
                .service(web::resource("/api/v1/manage/delete/{id}").route(web::delete().to(manage_delete_submission)))
                .service(web::resource("/api/v1/manage/user/delete/{user_id}").route(web::delete().to(manage_delete_user)))
                .service(web::resource("/api/v1/manage/user/team_admin_delete/{user_id}").route(web::delete().to(manage_delete_user_team_admin)))
                .service(web::resource("/api/v1/manage/access_key/delete/{access_key_id}").route(web::delete().to(manage_delete_access_key)))
                // PATCH
                .service(web::resource("/api/v1/manage/user/update_admin/{user_id}/{admin}").route(web::patch().to(manage_patch_admin)))
                .service(web::resource("/api/v1/manage/user/update_team_admin/{user_id}/{admin}").route(web::patch().to(manage_patch_team_admin)))
                .service(web::resource("/api/v1/manage/user/update_points/{user_id}/{modify}").route(web::patch().to(manage_patch_points)))
                .service(web::resource("/api/v1/manage/access_key/update/{id}/{key}").route(web::patch().to(manage_patch_access_key)))
                // POST
                .service(web::resource("/api/v1/manage/access_key/create/{key}/{team}").route(web::post().to(manage_post_access_key)))
            /* user endpoints */
            /* casino endpoints */
                // GET
                .service(web::resource("/api/v1/casino/spin_thing").route(web::get().to(casino_wheel)))
                .service(web::resource("/api/v1/casino/blackjack").route(web::get().to(casino::websocket_route)))
            /* points endpoints */
                // GET
                .service(web::resource("/api/v1/points/all").route(web::get().to(points_get_all)))
            /* misc endpoints */
                // GET
                .service(web::resource("/api/v1/transact/me").route(web::get().to(misc_get_transact_me)))
                .service(web::resource("/api/v1/whoami").route(web::get().to(misc_get_whoami)))
            /* debug endpoints */
                // GET
                .service(web::resource("/api/v1/debug/user").route(web::get().to(debug_get_user)))
    })
    .bind_openssl(format!("{}:443", env::var("HOSTNAME").unwrap_or_else(|_| "localhost".to_string())), builder)?
    .bind((env::var("HOSTNAME").unwrap_or_else(|_| "localhost".to_string()), 80))?
    .workers(2)
    .run()
    .await
}