use chrono::{
	DateTime,
	Utc,
};

use std::{
	env,
	time::SystemTime,
	convert::Infallible,
	net::SocketAddr,
};

use http_body_util::{
	combinators::BoxBody,
	BodyExt,
	Full,
};

use hyper::{
	body::{
		Bytes,
		Frame,
	},
	server::conn::http1,
	service::service_fn,
	header::HeaderValue,
	Request,
	Response,
	Method,
	StatusCode,
};

use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
	let addr = SocketAddr::from(([127, 0, 0, 1], 6980));

	let listener = TcpListener::bind(addr).await?;

	loop {
		let (stream, addr) = listener.accept().await?;

		let io = TokioIo::new(stream);

		let service = service_fn(move |req| {
			let addr = addr.clone();
			async move {
				Ok::<Response<Full<Bytes>>, Infallible>(serve(req, addr).await)
			}
		});

		tokio::task::spawn(async move {
			if let Err(err) = http1::Builder::new()
				.serve_connection(io, service)
				.await {
					println!("Error: {:?}", err);
				}
		});
	}
}

async fn serve(req: Request<hyper::body::Incoming>, addr: SocketAddr) -> Response<Full<Bytes>> {
	match (req.method(), req.uri().path()) {
		(&Method::POST, "/contact") => {
			Response::new(Full::new(Bytes::from(format!("{}", addr))))
		},
		_ => {
			let mut resp = Response::new(Full::new(Bytes::from("test")));
			resp.headers_mut().insert("Location", HeaderValue::from_static("https://gtohacks.github.io/website"));
			*resp.status_mut() = StatusCode::FOUND;
			resp
		},
	}
}

async fn post_comment(name: &str, title: &str, message: &str, email: &str, ipaddr: &str) -> Result<(), Box<dyn std::error::Error>> {
	let uri: &str = &env::var("WH_URI")?;

	let now = SystemTime::now();
	let now: DateTime<Utc> = now.into();
	let now: &str = &now.to_rfc3339();
	
	let params = &serde_json::json!({
		"content": "new message!!! (totally)",
		"username": name,
		"embeds": [
			{
				"title": title,
				"color": 0x2b2d31,
				"fields": [
					{
						"name": "Email",
						"value": email,
					},
					{
						"name": "IP Address",
						"value": format!("||{ipaddr}||"),
					},
				],
				"description": message,
				"timestamp": now,
			},
		],
	});

	let client = reqwest::Client::new();

	println!("{:#?}", params);

	let resp = client.post(uri)
		.json(params)
		.send()
		.await?;/*
		.text()
		.await?;
	*/
	println!("{resp:#?}");
	println!("{:#?}", resp.text().await?);
	Ok(())
}