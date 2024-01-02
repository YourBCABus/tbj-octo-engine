use graphql_client::{GraphQLQuery, Response};

use self::teachers::ResponseData;

// The paths are relative to the directory where your `Cargo.toml` is located.
// Both json and the GraphQL schema language are supported as sources for the schema

/// This makes the `GraphQLQuery` derive happy
#[allow(clippy::upper_case_acronyms)]
type UUID = uuid::Uuid;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/gql/definitions/schema.json",
    query_path = "src/gql/definitions/teachers.graphql",
    response_derives = "Debug,Clone,PartialEq"
)]

pub struct Teachers;

// TODO: Give this a better error variant
pub async fn fetch_teachers(variables: teachers::Variables) -> Result<ResponseData, ()> {
    // this is the important line
    let request_body = Teachers::build_query(variables);

    let client = reqwest::Client::new();
    let res = match client.post(dotenvy::var("GRAPHQL_ENDPOINT_URL").unwrap()).json(&request_body).send().await {
        Ok(res) => res,
        Err(e) => {
            println!("Error sending request: {}", e);
            return Err(());
        }
    };
    let response_body: Response<teachers::ResponseData> = match res.json().await {
        Ok(body) => body,
        Err(e) => {
            println!("Error parsing response body: {}", e);
            return Err(());
        }
    };

    // println!("{:#?}", response_body);
    match response_body.data {
        Some(data) => {
            Ok(data)
        },
        None => {
            println!("No data found");
            println!("{:#?}", response_body);
            Err(())
        }
    }
}
