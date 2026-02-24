#![cfg(test)]

use super::*;
use access_control::{role_admin, role_oracle, role_settlement_operator};
use soroban_sdk::{
    testutils::{Address as _, BytesN as _},
    Address, BytesN, Env, String, Symbol,
};

fn setup_payment_processor(env: &Env) -> PaymentProcessorClient<'_> {
    let contract_id = env.register(PaymentProcessor, ());
    PaymentProcessorClient::new(env, &contract_id)
}

fn setup_refund_manager(env: &Env) -> (Address, RefundManagerClient<'_>) {
    let contract_id = env.register(RefundManager, ());
    let client = RefundManagerClient::new(env, &contract_id);
    let admin = Address::generate(env);
    client.initialize_refund_manager(&admin);
    (admin, client)
}

#[test]
fn test_create_payment() {
    let env = Env::default();
    let client = setup_payment_processor(&env);

    let payment_id = String::from_str(&env, "payment_123");
    let merchant_id = Address::generate(&env);
    let amount = 1000000000i128; // 1000 USDC (6 decimals)
    let currency = Symbol::new(&env, "USDC");
    let deposit_address = Address::generate(&env);
    let expires_at = env.ledger().timestamp() + 3600;

    let payment = client.create_payment(
        &payment_id,
        &merchant_id,
        &amount,
        &currency,
        &deposit_address,
        &expires_at,
    );

    assert_eq!(payment.payment_id, payment_id);
    assert_eq!(payment.merchant_id, merchant_id);
    assert_eq!(payment.amount, amount);
    assert_eq!(payment.currency, currency);
    assert_eq!(payment.deposit_address, deposit_address);
    assert_eq!(payment.status, PaymentStatus::Pending);
}

#[test]
fn test_verify_payment_success() {
    let env = Env::default();
    let client = setup_payment_processor(&env);

    let payment_id = String::from_str(&env, "payment_123");
    let merchant_id = Address::generate(&env);
    let amount = 1000000000i128;
    let expires_at = env.ledger().timestamp() + 3600;

    client.create_payment(
        &payment_id,
        &merchant_id,
        &amount,
        &Symbol::new(&env, "USDC"),
        &Address::generate(&env),
        &expires_at,
    );

    let payer_address = Address::generate(&env);
    let transaction_hash = BytesN::<32>::random(&env);

    let status = client.verify_payment(&payment_id, &transaction_hash, &payer_address, &amount);

    assert_eq!(status, PaymentStatus::Confirmed);
    let payment = client.get_payment(&payment_id);
    assert_eq!(payment.status, PaymentStatus::Confirmed);
}

#[test]
fn test_create_and_get_refund() {
    let env = Env::default();
    let (_, client) = setup_refund_manager(&env);

    let payment_id = String::from_str(&env, "payment_123");
    let refund_amount = 1000i128;
    let reason = String::from_str(&env, "Reason");
    let requester = Address::generate(&env);

    let refund_id = client.create_refund(&payment_id, &refund_amount, &reason, &requester);
    let refund = client.get_refund(&refund_id);

    assert_eq!(refund.payment_id, payment_id);
    assert_eq!(refund.amount, refund_amount);
    assert_eq!(refund.status, RefundStatus::Pending);
}

#[test]
fn test_process_refund() {
    let env = Env::default();
    let (admin, client) = setup_refund_manager(&env);

    let payment_id = String::from_str(&env, "payment_123");
    let refund_amount = 1000i128;
    let requester = Address::generate(&env);
    let refund_id = client.create_refund(
        &payment_id,
        &refund_amount,
        &String::from_str(&env, "Reason"),
        &requester,
    );

    let operator = Address::generate(&env);
    client.grant_role(&admin, &role_settlement_operator(&env), &operator);

    client.process_refund(&operator, &refund_id);

    let refund = client.get_refund(&refund_id);
    assert_eq!(refund.status, RefundStatus::Completed);
}

#[test]
fn test_initialize_contract() {
    let env = Env::default();
    let admin = Address::generate(&env);

    let contract_id = env.register(RefundManager, ());
    let client = RefundManagerClient::new(&env, &contract_id);
    client.initialize_refund_manager(&admin);

    assert_eq!(client.get_admin(), Some(admin.clone()));
    assert!(client.has_role(&role_admin(&env), &admin));
}

#[test]
fn test_grant_role() {
    let env = Env::default();
    let (admin, client) = setup_refund_manager(&env);
    let account = Address::generate(&env);
    let role = role_oracle(&env);

    client.grant_role(&admin, &role, &account);
    assert!(client.has_role(&role, &account));
}

#[test]
fn test_transfer_admin() {
    let env = Env::default();
    let (current_admin, client) = setup_refund_manager(&env);
    let new_admin = Address::generate(&env);

    client.transfer_admin(&current_admin, &new_admin);

    assert!(client.has_role(&role_admin(&env), &new_admin));
    assert_eq!(client.get_admin(), Some(new_admin));
}
