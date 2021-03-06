indy::pool::Pool::set_protocol_version(PROTOCOL_VERSION).unwrap();

println!("1. Creating a new local pool ledger configuration that can be used later to connect pool nodes");
let pool_config_file = create_genesis_txn_file_for_pool(pool_name);
let pool_config = json!({
        "genesis_txn" : &pool_config_file
    });
Pool::create_ledger_config(&pool_name, Some(&pool_config.to_string())).unwrap();

println!("2. Open pool ledger and get the pool handle from libindy");
let pool_handle: i32 = Pool::open_ledger(&pool_name, None).unwrap();

println!("3. Creates a new wallet");
let config = json!({ "id" : wallet_name.to_string() }).to_string();
Wallet::create(&config, USEFUL_CREDENTIALS).unwrap();

println!("4. Open wallet and get the wallet handle from libindy");
let wallet_handle: i32 = Wallet::open(&config, USEFUL_CREDENTIALS).unwrap();

println!("5. Generating and storing steward DID and Verkey");
let first_json_seed = json!({
"seed":"000000000000000000000000Steward1"
}).to_string();
let (steward_did, _steward_verkey) = Did::new(wallet_handle, &first_json_seed).unwrap();

println!("6. Generating and storing Trust Anchor DID and Verkey");
let (trustee_did, trustee_verkey) = Did::new(wallet_handle, &"{}".to_string()).unwrap();

println!("7. Build NYM request to add Trust Anchor to the ledger");
let build_nym_request: String = Ledger::build_nym_request(&steward_did, &trustee_did, Some(&trustee_verkey), None, Some("TRUST_ANCHOR")).unwrap();

println!("8. Sending the nym request to ledger");
let _build_nym_sign_submit_result: String = Ledger::sign_and_submit_request(pool_handle, wallet_handle, &steward_did, &build_nym_request).unwrap();

println!("9. Create Schema and Build the SCHEMA request to add new schema to the ledger as a Steward");
let name = "gvt";
let version = "1.0";
let attributes = r#"["age", "sex", "height", "name"]"#;
let (schema_id, schema_json) = Issuer::create_schema(&steward_did, name, version, attributes).unwrap();

let build_schema_request: String = Ledger::build_schema_request(&steward_did, &schema_json).unwrap();

println!("10. Sending the SCHEMA request to the ledger");
let _signed_schema_request_response = Ledger::sign_and_submit_request(pool_handle, wallet_handle, &steward_did, &build_schema_request).unwrap();

println!("11. Creating and storing CREDENTIAL DEFINITION using anoncreds as Trust Anchor, for the given Schema");
let config_json = r#"{ "support_revocation": false }"#;
let tag = r#"TAG1"#;

let (cred_def_id, cred_def_json) = Issuer::create_and_store_credential_def(wallet_handle, &trustee_did, &schema_json, tag, None, config_json).unwrap();

println!("12. Creating Prover wallet and opening it to get the handle");
let prover_did = "VsKV7grR1BUE29mG2Fm2kX";
let prover_wallet_name = "prover_wallet";
let prover_wallet_config = json!({ "id" : prover_wallet_name.to_string() }).to_string();
Wallet::create(&prover_wallet_config, USEFUL_CREDENTIALS).unwrap();
let prover_wallet_handle: i32 = Wallet::open(&prover_wallet_config, USEFUL_CREDENTIALS).unwrap();

println!("13. Prover is creating Master Secret");
let master_secret_name = "master_secret";
Prover::create_master_secret(prover_wallet_handle, Some(master_secret_name)).unwrap();

println!("14. Issuer (Trust Anchor) is creating a Credential Offer for Prover");
let cred_offer_json = Issuer::create_credential_offer(wallet_handle, &cred_def_id).unwrap();

println!("15. Prover creates Credential Request");
let (cred_req_json, cred_req_metadata_json) = Prover::create_credential_req(prover_wallet_handle, prover_did, &cred_offer_json, &cred_def_json, &master_secret_name).unwrap();

println!("16. Issuer (Trust Anchor) creates Credential for Credential Request");

let cred_values_json = json!({
        "sex": { "raw": "male", "encoded": "5944657099558967239210949258394887428692050081607692519917050011144233115103" },
        "name": { "raw": "Alex", "encoded": "99262857098057710338306967609588410025648622308394250666849665532448612202874" },
        "height": { "raw": "175", "encoded": "175" },
        "age": { "raw": "28", "encoded": "28" },
    });

println!("cred_values_json = '{}'", &cred_values_json.to_string());

let (cred_json, _cred_revoc_id, _revoc_reg_delta_json) =
Issuer::create_credential(wallet_handle, &cred_offer_json, &cred_req_json, &cred_values_json.to_string(), None, -1).unwrap();

println!("17. Prover processes and stores Credential");
let _out_cred_id = Prover::store_credential(prover_wallet_handle, None, &cred_req_metadata_json, &cred_json, &cred_def_json, None).unwrap();
