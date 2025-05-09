use std::collections::BTreeMap;

use bls12_381::{G2Projective, Scalar};
use criterion::{Criterion, criterion_group, criterion_main};
use fedimint_core::encoding::{Decodable, Encodable};
use group::Curve;
use group::ff::Field;
use rand::rngs::OsRng;
use tbs::{
    AggregatePublicKey, BlindedSignatureShare, BlindingKey, Message, PublicKeyShare,
    SecretKeyShare, Signature, aggregate_signature_shares, blind_message, sign_message,
    unblind_signature, verify,
};

fn dealer_keygen(
    threshold: usize,
    keys: usize,
) -> (AggregatePublicKey, Vec<PublicKeyShare>, Vec<SecretKeyShare>) {
    let mut rng = OsRng;
    let poly: Vec<Scalar> = (0..threshold).map(|_| Scalar::random(&mut rng)).collect();

    let apk = (G2Projective::generator() * eval_polynomial(&poly, &Scalar::zero())).to_affine();

    let sks: Vec<SecretKeyShare> = (0..keys)
        .map(|idx| SecretKeyShare(eval_polynomial(&poly, &Scalar::from(idx as u64 + 1))))
        .collect();

    let pks = sks
        .iter()
        .map(|sk| PublicKeyShare((G2Projective::generator() * sk.0).to_affine()))
        .collect();

    (AggregatePublicKey(apk), pks, sks)
}

fn eval_polynomial(coefficients: &[Scalar], x: &Scalar) -> Scalar {
    coefficients
        .iter()
        .cloned()
        .rev()
        .reduce(|acc, coefficient| acc * x + coefficient)
        .expect("We have at least one coefficient")
}

fn bench_blinding(c: &mut Criterion) {
    c.bench_function("blinding", |b| {
        b.iter(|| {
            let msg = Message::from_bytes(b"Hello World!");
            let bkey = BlindingKey::random();
            blind_message(msg, bkey)
        })
    });
}

fn bench_signing(c: &mut Criterion) {
    let msg = Message::from_bytes(b"Hello World!");
    let bkey = BlindingKey::random();
    let bmsg = blind_message(msg, bkey);
    let (_pk, _pks, sks) = dealer_keygen(4, 5);

    c.bench_function("signing", |b| b.iter(|| sign_message(bmsg, sks[0])));
}

fn bench_aggregate(c: &mut Criterion) {
    let msg = Message::from_bytes(b"Hello World!");
    let bkey = BlindingKey::random();
    let bmsg = blind_message(msg, bkey);
    let (_pk, _pks, sks) = dealer_keygen(4, 5);
    let shares: BTreeMap<u64, BlindedSignatureShare> = (0_u64..)
        .zip(sks.iter().map(|sk| sign_message(bmsg, *sk)))
        .take(4)
        .collect();

    c.bench_function("signature aggregation", |b| {
        b.iter(|| aggregate_signature_shares(&shares))
    });
}

fn bench_unblind(c: &mut Criterion) {
    let msg = Message::from_bytes(b"Hello World!");
    let bkey = BlindingKey::random();
    let bmsg = blind_message(msg, bkey);
    let (_pk, _pks, sks) = dealer_keygen(4, 5);
    let shares = (0_u64..)
        .zip(sks.iter().map(|sk| sign_message(bmsg, *sk)))
        .take(4)
        .collect();
    let bsig = aggregate_signature_shares(&shares);

    c.bench_function("signature unblinding", |b| {
        b.iter(|| unblind_signature(bkey, bsig))
    });
}

fn bench_verify(c: &mut Criterion) {
    let msg = Message::from_bytes(b"Hello World!");
    let bkey = BlindingKey::random();
    let bmsg = blind_message(msg, bkey);
    let (pk, _pks, sks) = dealer_keygen(4, 5);
    let shares = (0_u64..)
        .zip(sks.iter().map(|sk| sign_message(bmsg, *sk)))
        .take(4)
        .collect();
    let bsig = aggregate_signature_shares(&shares);
    let sig = unblind_signature(bkey, bsig);

    c.bench_function("signature verification", |b| {
        b.iter(|| verify(msg, sig, pk))
    });
}

fn bench_decode_signature(c: &mut Criterion) {
    let msg = Message::from_bytes(b"Hello World!");
    let bkey = BlindingKey::random();
    let bmsg = blind_message(msg, bkey);
    let (_pk, _pks, sks) = dealer_keygen(4, 5);
    let shares = (1_u64..)
        .zip(sks.iter().map(|sk| sign_message(bmsg, *sk)))
        .take(4)
        .collect();
    let bsig = aggregate_signature_shares(&shares);
    let sig = unblind_signature(bkey, bsig);
    let sig_bytes = sig.consensus_encode_to_vec();

    c.bench_function("signature decoding", |b| {
        b.iter(|| {
            Signature::consensus_decode_whole(&sig_bytes, &Default::default())
                .expect("Decoding works")
        })
    });
}

criterion_group!(
    benches,
    bench_blinding,
    bench_signing,
    bench_aggregate,
    bench_unblind,
    bench_verify,
    bench_decode_signature
);
criterion_main!(benches);
