#![cfg_attr(not(feature = "export-abi"), no_main)]
#![cfg_attr(not(feature = "export-abi"), no_std)]
extern crate alloc;

use alloc::vec::Vec;
use alloc::vec;

use lazy_static::lazy_static;
use stylus_sdk::alloy_primitives::address;
use stylus_sdk::{alloy_primitives::U256, call::RawCall};
use stylus_sdk::prelude::*;
use substrate_bn::{AffineG1, Fq, Fr, G1};

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

const STATIC_CALL_GAS_PRICE: u64 = 2000000;

lazy_static! {
    // Scalar field size
    static ref R: U256 = "21888242871839275222246405745257275088548364400416034343698204186575808495617".parse().unwrap();
    // Base field size
    static ref Q: U256 = "21888242871839275222246405745257275088696311157297823662689037894645226208583".parse().unwrap();
    // Alpha
    static ref ALPHA_X: U256 = "6244961780046620888039345106890105326735326490660670538171427260567041582118".parse().unwrap();
    static ref ALPHA_Y: U256 = "9345530074574832515777964177156498988936486542424817298013748219694852051085".parse().unwrap();
    // Beta
    static ref BETA_X_1: U256 = "2818280727920019509567344333433040640814847647252965574434688845111015589444".parse().unwrap();
    static ref BETA_X_2: U256 = "2491450868879707184707638923318620824043077264425678122529022119991361101584".parse().unwrap();
    static ref BETA_Y_1: U256 = "5029766152948309994503689842780415913659475358303615599223648363828913323263".parse().unwrap();
    static ref BETA_Y_2: U256 = "2351008111262281888427337816453804537041498010110846693783231450896493019270".parse().unwrap();
    // Gamma
    static ref GAMMA_X_1: U256 = "11559732032986387107991004021392285783925812861821192530917403151452391805634".parse().unwrap();
    static ref GAMMA_X_2: U256 = "10857046999023057135944570762232829481370756359578518086990519993285655852781".parse().unwrap();
    static ref GAMMA_Y_1: U256 = "4082367875863433681332203403145435568316851327593401208105741076214120093531".parse().unwrap();
    static ref GAMMA_Y_2: U256 = "8495653923123431417604973247489272438418190587263600148770280649306958101930".parse().unwrap();
    // Delta
    static ref DELTA_X_1: U256 = "11559732032986387107991004021392285783925812861821192530917403151452391805634".parse().unwrap();
    static ref DELTA_X_2: U256 = "10857046999023057135944570762232829481370756359578518086990519993285655852781".parse().unwrap();
    static ref DELTA_Y_1: U256 = "4082367875863433681332203403145435568316851327593401208105741076214120093531".parse().unwrap();
    static ref DELTA_Y_2: U256 = "8495653923123431417604973247489272438418190587263600148770280649306958101930".parse().unwrap();
    // IC
    static ref IC_X: Vec<U256> = vec![

            "4257216062936355032264550010042049345445117955328839550223299514683519966016".parse().unwrap(),
        
            "17586290440796513468778571956491608579531524537588758492516276174418755874095".parse().unwrap(),
        
    ];
    static ref IC_Y: Vec<U256> = vec![
        
            "7771674729137698527856410080014061910651495679124913286228270043832624315711".parse().unwrap(),
        
            "12788400495420683658043651837043329035727378383150119505328920402810961381934".parse().unwrap(),
        
    ];
}

sol_storage! {
    #[entrypoint]
    pub struct StateTransitionVerifier {}
}

#[external]
impl StateTransitionVerifier {
    pub fn verify_proof(
        p_a_1: U256,
        p_a_2: U256,
        p_b_1_1: U256,
        p_b_1_2: U256,
        p_b_2_1: U256,
        p_b_2_2: U256,
        p_c_1: U256,
        p_c_2: U256,        
        signals: [U256; 1],
    ) -> Result<bool, Vec<u8>> {
        verify_proof([p_a_1, p_a_2], [[p_b_1_1, p_b_1_2], [p_b_2_1, p_b_2_2]], [p_c_1, p_c_2], signals)
    }
}

pub fn verify_proof(
    p_a: [U256; 2],
    p_b: [[U256; 2]; 2],
    p_c: [U256; 2],
    signals: [U256; 1],
) -> Result<bool, Vec<u8>> {
    for signal in signals.iter() {
        if *signal > *Q {
            return Ok(false);
        }
    }

    let mut vk = (*IC_X.get(0).unwrap(), *IC_Y.get(0).unwrap());
    for (i, signal) in signals.iter().enumerate() {
        vk = g1_mul_acc_c(
            vk,
            *IC_X.get(i + 1).unwrap(),
            *IC_Y.get(i + 1).unwrap(),
            *signal,
        );
    }

    let mut pairs: Vec<[u8; 192]> = Vec::new();

    let mut first_part = Vec::new();
    // -A
    first_part.extend_from_slice(p_a[0].to_be_bytes_vec().as_slice());
    first_part.extend_from_slice(((*Q - p_a[1]) % *Q).to_be_bytes_vec().as_slice());

    // B
    first_part.extend_from_slice(p_b[0][0].to_be_bytes_vec().as_slice());
    first_part.extend_from_slice(p_b[0][1].to_be_bytes_vec().as_slice());
    first_part.extend_from_slice(p_b[1][0].to_be_bytes_vec().as_slice());
    first_part.extend_from_slice(p_b[1][1].to_be_bytes_vec().as_slice());

    pairs.push(first_part.as_slice().try_into().unwrap());

    let mut second_part = Vec::new();
    // alpha1
    second_part.extend_from_slice((*ALPHA_X).to_be_bytes_vec().as_slice());
    second_part.extend_from_slice((*ALPHA_Y).to_be_bytes_vec().as_slice());

    // beta2
    second_part.extend_from_slice((*BETA_X_1).to_be_bytes_vec().as_slice());
    second_part.extend_from_slice((*BETA_X_2).to_be_bytes_vec().as_slice());
    second_part.extend_from_slice((*BETA_Y_1).to_be_bytes_vec().as_slice());
    second_part.extend_from_slice((*BETA_Y_2).to_be_bytes_vec().as_slice());

    pairs.push(second_part.as_slice().try_into().unwrap());

    let mut third_part = Vec::new();
    // vk
    third_part.extend_from_slice(vk.0.to_be_bytes_vec().as_slice());
    third_part.extend_from_slice(vk.1.to_be_bytes_vec().as_slice());

    // gamma2
    third_part.extend_from_slice((*GAMMA_X_1).to_be_bytes_vec().as_slice());
    third_part.extend_from_slice((*GAMMA_X_2).to_be_bytes_vec().as_slice());
    third_part.extend_from_slice((*GAMMA_Y_1).to_be_bytes_vec().as_slice());
    third_part.extend_from_slice((*GAMMA_Y_2).to_be_bytes_vec().as_slice());

    pairs.push(third_part.as_slice().try_into().unwrap());

    let mut fourth_part = Vec::new();
    // C
    fourth_part.extend_from_slice(p_c[0].to_be_bytes_vec().as_slice());
    fourth_part.extend_from_slice(p_c[1].to_be_bytes_vec().as_slice());

    // delta2
    fourth_part.extend_from_slice((*DELTA_X_1).to_be_bytes_vec().as_slice());
    fourth_part.extend_from_slice((*DELTA_X_2).to_be_bytes_vec().as_slice());
    fourth_part.extend_from_slice((*DELTA_Y_1).to_be_bytes_vec().as_slice());
    fourth_part.extend_from_slice((*DELTA_Y_2).to_be_bytes_vec().as_slice());

    pairs.push(fourth_part.as_slice().try_into().unwrap());

    let pairing = b128_pairing(pairs)?;

    Ok(pairing == U256::from(1))
}

fn g1_mul_acc_c(p_r: (U256, U256), x: U256, y: U256, s: U256) -> (U256, U256) {
    b128_add(b128_mul((x, y), s), p_r)
}

fn b258_point_unchecked(p: (U256, U256)) -> G1 {
    let x = Fq::from_slice(p.0.to_be_bytes_vec().as_slice()).unwrap();
    let y = Fq::from_slice(p.1.to_be_bytes_vec().as_slice()).unwrap();

    AffineG1::new(x, y).map(|v| G1::from(v)).unwrap()
}

fn b128_add(p1: (U256, U256), p2: (U256, U256)) -> (U256, U256) {
    let p1 = b258_point_unchecked(p1);
    let p2 = b258_point_unchecked(p2);

    let sum = AffineG1::from_jacobian(p1 + p2).unwrap();

    let mut x_bytes = [0u8; 32];
    let mut y_bytes = [0u8; 32];
    sum.x().to_big_endian(&mut x_bytes).unwrap();
    sum.y().to_big_endian(&mut y_bytes).unwrap();

    (U256::from_be_bytes(x_bytes), U256::from_be_bytes(y_bytes))
}

fn b128_mul(p: (U256, U256), fr: U256) -> (U256, U256) {
    let p = b258_point_unchecked(p);
    let fr = Fr::from_slice(fr.to_be_bytes_vec().as_slice()).unwrap();

    let product = AffineG1::from_jacobian(p * fr).unwrap();

    let mut x_bytes = [0u8; 32];
    let mut y_bytes = [0u8; 32];
    product.x().to_big_endian(&mut x_bytes).unwrap();
    product.y().to_big_endian(&mut y_bytes).unwrap();

    (U256::from_be_bytes(x_bytes), U256::from_be_bytes(y_bytes))
}

struct PairingContext {}

impl stylus_sdk::call::CallContext for PairingContext {
    fn gas(&self) -> u64 {
        STATIC_CALL_GAS_PRICE
    }
}

fn b128_pairing(pairs: Vec<[u8; 192]>) -> Result<U256, Vec<u8>> {
    let input = pairs.into_iter().flatten().collect::<Vec<u8>>();

    let output = RawCall::new_static()
        .gas(STATIC_CALL_GAS_PRICE)
        .call(address!("0000000000000000000000000000000000000008"), &input)?;

    Ok(U256::from_be_bytes::<32>(output.try_into().unwrap()))
}

#[deprecated]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_proof_test() {
        let p_a: [U256; 2] = [
            "18723443201642480319039599620596099377024417241726915870810579079003634371158".parse().unwrap(),
            "500740763351762952498603936950730862939440031657947624419462116048139885298".parse().unwrap(),
        ];
        let p_b: [[U256; 2]; 2] = [
            [
                "6223597660496502960355445119665299356894472222266227111076164637399528575607".parse().unwrap(),
                "5915640770752182162880783365226861626899303354012546973589690385910222314131".parse().unwrap(),
            ],
            [
                "1821882973281483810049572800109692442909218292042903909418300938743404335792".parse().unwrap(),
                "9788634418284081410006491714334804457142916454410638964952595212196991328432".parse().unwrap(),
            ]
        ];
        let p_c: [U256; 2] = [
            "19865471465847785801131588527778640700182140965012278394177417604184949553309".parse().unwrap(),
            "2298649547684658174100090253853510015792969090560932538645500114235289350432".parse().unwrap(),
        ];
        let signals: [U256; 1] = [
            "18630398846081570358266919481382955945076989170608567921689539672329067433281".parse().unwrap(),
        ];

        let is_verified = verify_proof(p_a, p_b, p_c, signals).unwrap();

        assert!(is_verified);
    }
}
