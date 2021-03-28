use crate::RedisPool;
use actix_web::{
  web::{self, ServiceConfig},
  Error, HttpMessage, HttpRequest, HttpResponse,
};
use juniper_actix::{graphiql_handler, graphql_handler, playground_handler};
use sqlx::PgPool;

use crate::{
  repositories::{
    ApiKeyRepository, MacAddressRepository, SessionRepository, StudyPeriodRepository,
    StudyYearRepository, UserRepository, UserSessionRepository,
  },
  schema::{Context, ContextRepositories, ContextServices, Schema},
  services::{stats::StatsService, user::UserService},
};

async fn playground() -> Result<HttpResponse, Error> {
  playground_handler("/graphql", None).await
}

async fn graphiql() -> Result<HttpResponse, Error> {
  graphiql_handler("/graphql", None).await
}

async fn graphql(
  req: HttpRequest,
  payload: web::Payload,
  schema: web::Data<Schema>,
  db_pool: web::Data<PgPool>,
  redis_pool: web::Data<RedisPool>,
) -> Result<HttpResponse, Error> {
  let db_pool = PgPool::clone(&db_pool);
  let redis_pool = RedisPool::clone(&redis_pool);
  let context = Context {
    repos: ContextRepositories {
      api_key: ApiKeyRepository::new(db_pool.clone()),
      mac_addr: MacAddressRepository::new(db_pool.clone()),
      session: SessionRepository::new(db_pool.clone()),
      study_period: StudyPeriodRepository::new(db_pool.clone()),
      study_year: StudyYearRepository::new(db_pool.clone()),
      user: UserRepository::new(),
      user_session: UserSessionRepository::new(db_pool.clone()),
    },
    services: ContextServices {
      stats: StatsService::new(
        UserSessionRepository::new(db_pool.clone()),
        StudyYearRepository::new(db_pool.clone()),
        StudyPeriodRepository::new(db_pool.clone()),
        redis_pool.clone(),
      ),
      user: UserService::new(UserRepository::new(), redis_pool.clone()),
    },
    headers: req.headers().clone(),
    cookies: req.cookies().map_or(vec![], |v| v.clone()),
    redis_pool,
  };
  graphql_handler(&schema, &context, req, payload).await
}

pub fn init(config: &mut ServiceConfig) {
  config
    .service(
      web::resource("/graphql")
        .route(web::get().to(playground))
        .route(web::post().to(graphql)),
    )
    .service(web::resource("/graphiql").route(web::get().to(graphiql)));
}