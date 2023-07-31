use actix_web::{cookie::Cookie, test, App};

use crate::{app_config::config_app, key_extractor::ErrorMessage};

#[actix_web::test]
#[ignore]
async fn test_post_vault() {
    let app = test::init_service(App::new().configure(config_app)).await;
    let auth1 = Cookie::new("Authorization", "Bearer A");
    let auth2 = Cookie::new("Authorization", "Bearer B");
    let auth3 = Cookie::new("Authorization", "Bearer C");
    // first 3 request should pass
    for _ in 0..3 {
        let req = test::TestRequest::post()
            .cookie(auth1.clone())
            .uri("/vault")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    // 4th request should error
    let req = test::TestRequest::post()
        .cookie(auth1.clone())
        .uri("/vault")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_client_error());
    let msg: ErrorMessage = test::read_body_json(resp).await;
    assert_eq!(msg.code, 429);
    assert!(msg.wait_time_ms > 0);

    // where another user with a different auth token should not be affected
    let req = test::TestRequest::post()
        .cookie(auth2)
        .uri("/vault")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    // sleep through the time
    let sleep_time = std::time::Duration::from_secs(60);
    std::thread::sleep(sleep_time);

    // after sleep, the rate limiter should reset and work again
    for _ in 0..3 {
        let req = test::TestRequest::post()
            .cookie(auth1.clone())
            .uri("/vault")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    // looping using 20s period to call the API. it should not have error as its within the limit
    for i in 0..4 {
        let req = test::TestRequest::post()
            .cookie(auth3.clone())
            .uri("/vault")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        if i != 3 {
            let sleep_time = std::time::Duration::from_secs(20);
            std::thread::sleep(sleep_time);
        }
    }
}

#[actix_web::test]
#[ignore]
async fn test_get_vault_items() {
    let app = test::init_service(App::new().configure(config_app)).await;
    let auth1 = Cookie::new("Authorization", "Bearer A");
    let auth2 = Cookie::new("Authorization", "Bearer B");
    let mut count = 0;
    // first 1200 request should pass
    for _ in 0..1300 {
        let req = test::TestRequest::get()
            .cookie(auth1.clone())
            .uri("/vault/items")
            .to_request();
        let resp = test::call_service(&app, req).await;
        if resp.status().is_success() {
            count += 1
        } else {
            let msg: ErrorMessage = test::read_body_json(resp).await;
            assert_eq!(msg.code, 429);
            assert!(msg.wait_time_ms > 0);
        }
    }
    // allow small buffer to offset the processing time, as this replenish fast
    assert!(count >= 1200);
    assert!(count < 1210);

    // where another user with a different auth token should not be affected
    let req = test::TestRequest::get()
        .cookie(auth2)
        .uri("/vault/items")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let sleep_time = std::time::Duration::from_secs(60);
    std::thread::sleep(sleep_time);

    let mut count = 0;
    // after sleep, the rate limiter should reset and work again
    for _ in 0..1200 {
        let req = test::TestRequest::get()
            .cookie(auth1.clone())
            .uri("/vault/items")
            .to_request();
        let resp = test::call_service(&app, req).await;
        if resp.status().is_success() {
            count += 1;
        }
    }
    // allow small buffer to offset the processing time, as this replenish fast
    assert!(count >= 1200);
    assert!(count < 1210);
}

#[actix_web::test]
#[ignore]
async fn test_put_vault_item() {
    let app = test::init_service(App::new().configure(config_app)).await;
    let auth1 = Cookie::new("Authorization", "Bearer A");
    let auth2 = Cookie::new("Authorization", "Bearer B");
    // first 60 request should pass
    for _ in 0..60 {
        let req = test::TestRequest::put()
            .cookie(auth1.clone())
            .uri("/vault/items/1")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    // request should error afterwards
    let req = test::TestRequest::put()
        .cookie(auth1.clone())
        .uri("/vault/items/1")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_client_error());
    let msg: ErrorMessage = test::read_body_json(resp).await;
    assert_eq!(msg.code, 429);
    assert!(msg.wait_time_ms > 0);

    // where another user with a different auth token and same path should not be affected
    let req = test::TestRequest::put()
        .cookie(auth2)
        .uri("/vault/items/1")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    // same auth token with another items should not be affected
    let req = test::TestRequest::put()
        .cookie(auth1.clone())
        .uri("/vault/items/2")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let sleep_time = std::time::Duration::from_secs(60);
    std::thread::sleep(sleep_time);

    // after sleep, the rate limiter should reset and work again
    for _ in 0..60 {
        let req = test::TestRequest::put()
            .cookie(auth1.clone())
            .uri("/vault/items/1")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }
}
