#[derive(Serialize, Deserialize)]
pub struct TransactionList {
    #[serde(rename = "type")]
    pub transaction_list_type: String,

    #[serde(rename = "employeeEmail")]
    pub employee_email: String,

    #[serde(rename = "transactionList")]
    pub transaction_list: Vec<TransactionListElement>,
}

#[derive(Serialize, Deserialize)]
pub struct TransactionListElement {
    #[serde(rename = "created")]
    pub created: String,

    #[serde(rename = "currency")]
    pub currency: String,

    #[serde(rename = "merchant")]
    pub merchant: String,

    #[serde(rename = "amount")]
    pub amount: i64,

    #[serde(rename = "category")]
    pub category: String,

    #[serde(rename = "tag")]
    pub tag: String,

    #[serde(rename = "billable")]
    pub billable: bool,

    #[serde(rename = "reimbursable")]
    pub reimbursable: bool,

    #[serde(rename = "comment")]
    pub comment: String,
}
