use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web::{error, middleware, web, App, Error, HttpResponse, HttpServer};
use env_logger::Env;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Mutex;

const MAX_SIZE: usize = 262_144; // max payload size is 256k

async fn health() -> Result<HttpResponse, Error> {
    let version = std::env::var("APP_VERSION").unwrap_or_else(|_| "0.1.0".to_string());
    let health = json!({"status": "ok", "version": version});
    Ok(HttpResponse::Ok().json(health))
}

async fn get_product(
    data: web::Data<AppState>,
    path: web::Path<ProductInfo>,
) -> Result<HttpResponse, Error> {
    let products = data.products.lock().unwrap();

    // find product by id in products
    let index = products
        .iter()
        .position(|p| p.id == path.product_id)
        .unwrap();

    Ok(HttpResponse::Ok().json(products[index].clone()))
}

async fn get_products(data: web::Data<AppState>) -> Result<HttpResponse, Error> {
    let products = data.products.lock().unwrap();
    Ok(HttpResponse::Ok().json(products.to_vec()))
}

async fn add_product(
    data: web::Data<AppState>,
    mut payload: web::Payload,
) -> Result<HttpResponse, Error> {
    let mut products = data.products.lock().unwrap();
    let new_id = products.len() as i32 + 1;

    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let mut product = serde_json::from_slice::<Product>(&body)?;

    // update product id
    product.id = new_id;

    // add product to products
    products.push(product.clone());

    Ok(HttpResponse::Ok().json(product))
}

async fn update_product(
    data: web::Data<AppState>,
    mut payload: web::Payload,
) -> Result<HttpResponse, Error> {
    let mut products = data.products.lock().unwrap();

    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let product = serde_json::from_slice::<Product>(&body)?;

    // replace product with same id
    let index = products.iter().position(|p| p.id == product.id).unwrap();
    products[index] = product.clone();

    Ok(HttpResponse::Ok().json(product))
}

async fn delete_product(
    data: web::Data<AppState>,
    path: web::Path<ProductInfo>,
) -> Result<HttpResponse, Error> {
    let mut products = data.products.lock().unwrap();

    // find product by id in products
    let index = products
        .iter()
        .position(|p| p.id == path.product_id)
        .unwrap();

    // remove product from products
    products.remove(index);

    Ok(HttpResponse::Ok().body(""))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let products = vec![
        Product {
            id: 1,
            name: "Smart Thermostat".to_string(),
            price: 199.99,
            description: "A smart thermostat allows you to remotely control and schedule your home's heating and cooling systems through a smartphone app. It can also learn your preferences and adjust the temperature accordingly.".to_string(),
            image: "https://www.bhphotovideo.com/images/images2500x2500/google_ga02082_us_nest_thermostat_sand_1597181.jpg".to_string()
        },
        Product {
            id: 2,
            name: "Connected Fitness Tracker".to_string(),
            price: 169.99,
            description: "A connected fitness tracker measures your physical activity, heart rate, and sleep patterns. It syncs with your smartphone to provide real-time health and fitness data and insights.".to_string(),
            image: "https://p.globalsources.com/IMAGES/PDT/B5696870693/Smart-Bracelet.jpg".to_string()
        },
        Product {
            id: 3,
            name: "Smart Doorbell".to_string(),
            price: 129.99,
            description: "This IoT gadget includes a camera and intercom system, enabling you to see and communicate with visitors at your front door through a mobile app, even when you're not at home.".to_string(),
            image: "https://m.media-amazon.com/images/I/71aSgA2f7-S._AC_UF894,1000_QL80_.jpg".to_string()
        },
        Product {
            id: 4,
            name: "Wireless Smart Lighting".to_string(),
            price: 118.99,
            description: "These smart light bulbs and switches can be controlled remotely, dimmed, and scheduled using a smartphone app. Some models can change colors and sync with music or your TV.".to_string(),
            image: "https://qa1aerocartnet.s3.eu-central-1.amazonaws.com/uploads/fashion_store_staging/categories/Smart%20Lights_20221009910B7.jpg".to_string()
        },
        Product {
            id: 5,
            name: "Smart Lock".to_string(),
            price: 256.99,
            description: "A smart lock provides keyless entry to your home using a mobile app or even voice commands. It offers enhanced security features, including guest access control.".to_string(),
            image: "https://images.thdstatic.com/productImages/8a7aec4e-3740-40b3-a461-be81be7e1928/svn/ultraloq-electronic-locksets-ubpbhb-64_600.jpg".to_string()
        },
        Product {
            id: 6,
            name: "IOT Pet Feeder".to_string(),
            price: 140.99,
            description: "This device lets you feed your pets remotely and on a schedule, ensuring your furry friends are well-fed even when you're away from home.".to_string(),
            image: "https://media.karousell.com/media/photos/products/2020/11/16/automatic_pet_feeder_battery___1605488144_b9fcf149.jpg".to_string()
        },
        Product {
            id: 7,
            name: "Smart Garden Sensors".to_string(),
            price: 19.99,
            description: "IoT garden sensors monitor soil moisture, light levels, and temperature, sending real-time data to your phone. They help you optimize plant care and conserve water.".to_string(),
            image: "https://www.deliacreates.com/wp-content/uploads/2015/07/Edyn-30-of-510717.jpg".to_string()
        },
        Product {
            id: 8,
            name: "IoT Coffee Maker".to_string(),
            price: 507.99,
            description: "This smart coffee maker can be programmed to brew your favorite coffee remotely through a smartphone app. You can even adjust the strength and brewing time.".to_string(),
            image: "https://media.techeblog.com/images/smarter-coffee.jpg".to_string()
        },
        Product {
            id: 9,
            name: "Conected baby monitor".to_string(),
            price: 53.99,
            description: "A connected baby monitor offers video and audio streaming to your smartphone, providing peace of mind by keeping an eye on your baby, even from another room or location.".to_string(),
            image: "https://www.clement.ca/media/catalog/product/H/U/HUB-HCSNPCL2-CA_A.jpg".to_string()
        },
        Product {
            id: 10,
            name: "Smart Refrigerator".to_string(),
            price: 1505.99,
            description: "This IoT gadget features a touchscreen display on the door, allowing you to check the contents, create shopping lists, and even order groceries online. It can also suggest recipes based on available ingredients.".to_string(),
            image: "https://media.betterlifeuae.com/catalog/product/r/q/rq759n4ibu1-b.jpg".to_string()
        }
    ];

    let product_state = web::Data::new(AppState {
        products: Mutex::new(products.to_vec()),
    });

    println!("Listening on http://0.0.0.0:3002");

    env_logger::init_from_env(Env::default().default_filter_or("info"));

    HttpServer::new(move || {
        let cors = Cors::permissive();

        App::new()
            .wrap(cors)
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .wrap(middleware::DefaultHeaders::new().add(("X-Version", "0.2")))
            .app_data(product_state.clone())
            .route("/health", web::get().to(health))
            .route("/health", web::head().to(health))
            .route("/{product_id}", web::get().to(get_product))
            .route("/", web::get().to(get_products))
            .route("/", web::post().to(add_product))
            .route("/", web::put().to(update_product))
            .route("/{product_id}", web::delete().to(delete_product))
    })
    .bind(("0.0.0.0", 3002))?
    .run()
    .await
}

struct AppState {
    products: Mutex<Vec<Product>>,
}

#[derive(Serialize, Deserialize, Clone)]
struct Product {
    id: i32,
    name: String,
    price: f32,
    description: String,
    image: String,
}

#[derive(Deserialize)]
struct ProductInfo {
    product_id: i32,
}
