// @generated automatically by Diesel CLI.

diesel::table! {
    measurements (id) {
        id -> Integer,
        meas_time -> BigInt,
        temperature -> Nullable<Float>,
        humidity -> Nullable<Float>,
        pressure -> Nullable<Float>,
        light_level -> Nullable<Float>,
    }
}
