use crate::{
    config::AppConfig,
    create_rocket_instance,
    db::{self, test::DatabaseDropper},
    setup_rocket_instance,
};
use rocket::{Build, Rocket};
use std::path::PathBuf;
use uuid::Uuid;

/// Creates a new Rocket instance for testing.
/// It creates a new database for the test and runs the migrations.
pub async fn create_test_rocket_instance() -> (Rocket<Build>, DatabaseDropper) {
    let mut app_config = AppConfig::load(None as Option<PathBuf>).unwrap();

    let database_url_base = app_config.database_url_base.clone();
    let maintenance_database_name = app_config.maintenance_database_name.clone();
    let id = Uuid::new_v4().to_string();

    let database_name =
        db::test::create_test_database(&database_url_base, &maintenance_database_name, &id)
            .unwrap();
    app_config.database_name = database_name.clone();

    let rocket = create_rocket_instance(&app_config).unwrap();
    let rocket = setup_rocket_instance(app_config, rocket, false)
        .await
        .unwrap();
    let database_dropper = DatabaseDropper::new(
        &database_url_base,
        &maintenance_database_name,
        &database_name,
    );

    (rocket, database_dropper)
}

pub mod helpers {
    use rocket::{
        http::{Accept, ContentType, Header},
        local::asynchronous::Client,
    };

    use crate::{
        db::models::{StagingFile, User, UserSession},
        services::{AuthService, StagingFileService, UserService},
    };

    pub async fn create_user(id: &str, user_service: &UserService) -> User {
        let user = user_service
            .create_user(
                &format!("{}_user", id),
                &format!("{}_user@example.com", id),
                &format!("{}_user_pw", id),
            )
            .await
            .unwrap();
        user
    }

    pub async fn create_initial_user(
        auth_service: &AuthService,
        user_service: &UserService,
    ) -> (User, UserSession) {
        let user = create_user("initial", user_service).await;
        let user_session = auth_service.create_user_session(user.id).await.unwrap();
        (user, user_session)
    }

    pub async fn create_filled_staging_file(
        client: &Client,
        staging_file_service: &StagingFileService,
        user_session: &UserSession,
        name: impl AsRef<str>,
        mime: Option<impl AsRef<str>>,
        file_content: impl AsRef<[u8]>,
    ) -> StagingFile {
        let name = name.as_ref();
        let mime = mime.as_ref().map(|mime| mime.as_ref());

        let staging_file = staging_file_service
            .create_staging_file(name, mime)
            .await
            .unwrap();

        let file_content = file_content.as_ref();

        let response = client
            .put(format!("/staging-files/{}", staging_file.id))
            .header(Accept::JSON)
            .header(ContentType::Binary)
            .header(Header::new(
                "Authorization",
                format!("Bearer {}", user_session.token),
            ))
            .body(file_content)
            .dispatch()
            .await;

        let filled_staging_file = response.into_json::<StagingFile>().await.unwrap();

        filled_staging_file
    }
}
