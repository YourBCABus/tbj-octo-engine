use graphql_client::{GraphQLQuery, Response};

use crate::env::GRAPHQL_ENDPOINT_URL;

use self::period_teacher::ResponseData;

use super::post_req_auth;

/// This makes the `GraphQLQuery` derive happy
#[allow(clippy::upper_case_acronyms)]
type UUID = uuid::Uuid;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/gql/definitions/schema.graphql",
    query_path = "src/gql/definitions/period_teacher.graphql",
    response_derives = "Debug,Clone,PartialEq"
)]

pub struct PeriodTeacher;

// TODO: Give this a better error variant
pub async fn fetch_teacher_periods(variables: period_teacher::Variables) -> Result<ResponseData, ()> {
    // this is the important line
    let request_body = PeriodTeacher::build_query(variables);

    let res = match post_req_auth(&GRAPHQL_ENDPOINT_URL).json(&request_body).send().await {
        Ok(res) => res,
        Err(e) => {
            println!("Error sending request: {}", e);
            return Err(());
        }
    };
    let response_body: Response<period_teacher::ResponseData> = match res.json().await {
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
