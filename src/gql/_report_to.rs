use graphql_client::{GraphQLQuery, Response};

use self::report_to::ResponseData;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/gql/definitions/schema.graphql",
    query_path = "src/gql/definitions/report_to.graphql",
    response_derives = "Debug,Clone,PartialEq"
)]
pub struct ReportTo;

// TODO: Give this a better error variant
pub async fn fetch_report_to(variables: report_to::Variables) -> Result<ResponseData, ()> {
    // this is the important line
    let request_body = ReportTo::build_query(variables);

    let client = reqwest::Client::new();
    let res = match client.post(dotenvy::var("GRAPHQL_ENDPOINT_URL").unwrap()).json(&request_body).send().await {
        Ok(res) => res,
        Err(e) => {
            println!("Error sending request: {}", e);
            return Err(());
        }
    };
    let response_body: Response<ResponseData> = match res.json().await {
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
