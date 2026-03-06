// SPDX-FileCopyrightText: 2026 Greenbone AG
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use base64::Engine;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FeedKeyValidationError {
    #[error("Invalid Identifier")]
    InvalidIdentifier,

    #[error("Missing Private Key Start marker")]
    MissingPrivateKeyStartMarker,

    #[error("Missing Key data")]
    MissingKeyData,

    #[error("Invalid Key data")]
    InvalidKeyData,

    #[error("Missing Private Key End marker")]
    MissingPrivateKeyEndMarker,
}

pub trait FeedKeyValidationState: Send + Sync + std::fmt::Debug {
    fn push(&self, line: &str) -> Result<Box<dyn FeedKeyValidationState>, FeedKeyValidationError>;

    fn done(&self) -> Result<(), FeedKeyValidationError>;
}

pub trait FeedKeyValidator: Sync + Send {
    fn push(&mut self, line: &str) -> Result<(), FeedKeyValidationError>;

    fn done(&mut self) -> Result<(), FeedKeyValidationError>;
}

pub struct PlainFeedKeyValidator {
    state: Box<dyn FeedKeyValidationState>,
}

impl PlainFeedKeyValidator {
    pub fn new() -> Self {
        PlainFeedKeyValidator {
            state: Box::new(states::New {}),
        }
    }
}

impl FeedKeyValidator for PlainFeedKeyValidator {
    fn push(&mut self, line: &str) -> Result<(), FeedKeyValidationError> {
        let state = self.state.push(line)?;
        self.state = state;
        Ok(())
    }

    fn done(&mut self) -> Result<(), FeedKeyValidationError> {
        self.state.done()
    }
}

pub struct Base64FeedKeyValidator {
    validator: PlainFeedKeyValidator,
    current_line: String,
}

impl Base64FeedKeyValidator {
    pub fn new() -> Self {
        Base64FeedKeyValidator {
            validator: PlainFeedKeyValidator::new(),
            current_line: String::new(),
        }
    }
}

impl FeedKeyValidator for Base64FeedKeyValidator {
    fn push(&mut self, line: &str) -> Result<(), FeedKeyValidationError> {
        let decoded_line = base64::engine::general_purpose::STANDARD
            .decode(line.trim())
            .map(|bytes| String::from_utf8_lossy(&bytes).to_string())
            .map_err(|_| FeedKeyValidationError::InvalidKeyData)?;
        self.current_line.push_str(&decoded_line);
        let mut decoded_lines = self
            .current_line
            .lines()
            .map(|l| l.to_string())
            .collect::<Vec<String>>();
        self.current_line = decoded_lines.pop().unwrap_or_default(); // Keep the last line in case it's incomplete
        for line in decoded_lines {
            self.validator.push(&line)?;
        }
        Ok(())
    }

    fn done(&mut self) -> Result<(), FeedKeyValidationError> {
        self.validator.push(&self.current_line)?; // Process any remaining line
        self.validator.done()
    }
}

pub mod states {
    use super::*;

    #[derive(Debug)]
    pub struct New {}
    #[derive(Debug)]
    pub struct HasKeyIdentifier {}
    #[derive(Debug)]
    pub struct HasStartPrivateKey {}
    #[derive(Debug)]
    pub struct HasEndPrivateKey {}

    impl FeedKeyValidationState for New {
        fn push(
            &self,
            line: &str,
        ) -> Result<Box<dyn FeedKeyValidationState>, FeedKeyValidationError> {
            if !line.contains("@") {
                return Err(FeedKeyValidationError::InvalidIdentifier);
            }
            Ok(Box::new(HasKeyIdentifier {}))
        }

        fn done(&self) -> Result<(), FeedKeyValidationError> {
            Err(FeedKeyValidationError::MissingKeyData)
        }
    }

    impl FeedKeyValidationState for HasKeyIdentifier {
        fn push(
            &self,
            line: &str,
        ) -> Result<Box<dyn FeedKeyValidationState>, FeedKeyValidationError> {
            if !line.starts_with("-----BEGIN ") || !line.ends_with(" KEY-----") {
                return Err(FeedKeyValidationError::MissingPrivateKeyStartMarker);
            }
            Ok(Box::new(HasStartPrivateKey {}))
        }

        fn done(&self) -> Result<(), FeedKeyValidationError> {
            Err(FeedKeyValidationError::MissingPrivateKeyStartMarker)
        }
    }

    impl FeedKeyValidationState for HasStartPrivateKey {
        fn push(
            &self,
            line: &str,
        ) -> Result<Box<dyn FeedKeyValidationState>, FeedKeyValidationError> {
            if line.starts_with("-----END ") && line.ends_with(" KEY-----") {
                return Ok(Box::new(HasEndPrivateKey {}));
            } else if line.is_ascii() {
                return Ok(Box::new(HasStartPrivateKey {}));
            }
            Err(FeedKeyValidationError::InvalidKeyData)
        }

        fn done(&self) -> Result<(), FeedKeyValidationError> {
            Err(FeedKeyValidationError::MissingPrivateKeyEndMarker)
        }
    }

    impl FeedKeyValidationState for HasEndPrivateKey {
        fn push(
            &self,
            _line: &str,
        ) -> Result<Box<dyn FeedKeyValidationState>, FeedKeyValidationError> {
            Ok(Box::new(HasEndPrivateKey {}))
        }

        fn done(&self) -> Result<(), FeedKeyValidationError> {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_KEY_IDENTIFIER: &str = "gsf201809061@feed.greenbone.net:/feed/";
    const VALID_PRIVATE_KEY_START: &str = "-----BEGIN PRIVATE KEY-----";
    const VALID_PRIVATE_KEY_DATA: &str =
        "MIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQDCZsXjHqL";
    const VALID_PRIVATE_KEY_END: &str = "-----END PRIVATE KEY-----";

    const OLD_STAGING_KEY: &str = "
    ZG93bmxvYWRAc3RhZ2luZy5vcGVyYXRpb24ub3MuZ3JlZW5ib25lLm5ldDovZmVlZC8KLS0tLS1C
RUdJTiBSU0EgUFJJVkFURSBLRVktLS0tLQpNSUlFb1FJQkFBS0NBUUVBdSsvSXVMcjNaOXlyK1o5
d3dXcUtZVDN4RlZxTnFtUVo1QzVhVG9iZ3czNFdITThjClMzQXJGQm9McU5LMFlXUlNyS05XOGNY
R1BiVEV2Y0NKaFBWbWFVc0QzMHZBRVlzWExONklJYm5oT2dpdGVCbm0KekREVDVRZmZSTUdMUzYx
WU5RSnpoMjdEUWRrSjRBVkI3enBBVnRVdTNwaWkzZHQyZzBPR1pmd214L2hPS3kwUAoxRzhDOGFI
WXYvbUJpZ0NWdm9ScUhjWXFKZHJ6R3A4Ym8wVHkrK09XMjRtaTZvUUcxb2ZvdEh1dk02U1JUajky
Ckc2ZlFlZGF0OUNCQnhwbEdqc3JKUUZwOU9wUGVWRWVDNnNveGdUc1ZsZUtldXpRaUU3Y05iTG1j
TUEyYVBLRk4KTnpUUmpHbUc4dlp4TStpempBSFdNeG95b01JRVBkQjkzNGdOOHdJQkl3S0NBUUFR
Rzk0Qk5KamRCRWxCU0M0OApkdGlwUDlMVjhkR2c1QUk0SVRyd3lidCtlSVdOY05hS0g0NnA4NXFa
Y1NWbmJ2L0ZwNW04ODdIb0NDNGU1SjRTCnRmTFYwenJZb0JmRy9Vc2hHbU53bXVkclg5UmF3R1E5
WTBWY3hpa1VoWjQ1cjhXN1pwd3dMaEM4Z0ZGTnQwN0wKWE1PdnE5OXlLbGNhVktQQ0c3c1FEa3gz
aWlvRkUxYnhuanVFN1J1REdwWk1hVUNsWXl4V1ZtTjJYc281L1J3Kwo0c1lXUlhxZzZDSk12MHB1
ZzVDekVEWFpCalhBNmNWWnBydE1iLzFNQmhjTDhrOEZ5TUpyQ0dSV0w1d1BDcjVOCmtjMzFoTS8v
MDJnTk5SQ21pMHFhby9lbGhIMlBsU000S1hlY1NuRk5ER2JjL2IvNUNLREgwVkxldzRBMXZKb0YK
WDljakFvR0JBUFI0dmlXZ0FEZzFUdnp6WEsvUWgyWUZCQ2dyNWQvamlUTWV6Y0lVT0tpRENQeFZk
L0JwRFcyQgo1czFCMkpVVkNIbGtKVm03VUcyemdSOEhCcWYrVHNRZWE1QUN6WU9iZ1dpK0VMMWIw
UFdkQUxsVldtZXNrbllVCnFpZ3JlbnVEejVHOFFCL1IzdjZPMi8xSGg3ZzBWUTFPQkkyVncwemFQ
QW9xbEtESWdjcWxBb0dCQU1UTWpVa3gKamhSWUwwVjl1L2crQXBMYXBlZFg4R3lILzk0Qjh4QUd0
cGpLV3E2R0tDSlhhWXBMVE5pZTVmczZxUG9mY3NpZgp2aTcxT3dQTGZUZVNQNU1zNDdKYzFieGNQ
RmptdVc2d3FZT3oxNHhDa0hrZk5wUm9DU21LY05kMmlhVDFQK2ZXCkhMVGVLUXA5M2dqdFdKRTU3
ZlRYZDNSS1I4ejFvNExZVXNxM0FvR0FXczNDK0FnNm1JalVNaEUvcndSUGpFc0IKaTBPQlJJZTJu
ZlYvbjl1Z0JCTnB2TTlKemxMbnV2VzhJRnBKSVc0MldQbFBzNTFZWTBLc1RWcGhqdHJNeklCVAox
bWR3NTcxcWxKNWQvVEM3VStuYnNvMXF1TXNSMUJaTjFHZnJxamhGeUdwK043UmFKQWt0STQrWXoy
UGxFNUlCCnNPNTd2WFdoUlpySmZZeE5keWNDZ1lFQWh2S3FCazNkeE03QlZEajE3QTFEbCtaaklr
cnVBVWRRWGJneHBKYmoKbS9oNHNqQWJoVU5CRGwrRkp0TmIzM2l1WWw2MUc5dEFsVHB4bkRQRGs5
SC90V0NjSW9qTkUzSnloaHFOeUQ2ZwpIOHZIQVJlemhrRkovMTFJSEh3d0hyZXUxNE9aaVFmWks1
RUdNeFI2L3M2akljRlN0b1VldGlST2ZlcEVQSGNVCk16c0NnWUJ3L1NXRmVGZzZGeGtEbU1uUzNx
bEJ5MlMyRG1NQ24xSkJvTHRCc1VvcmR2S3lnbHhPTEpGOWlvSDUKOHpGYTdKUFF4NlpVbm83cTJl
dkMxcU9IMm8wRUhFQklHb1dGNTdlM2pFNDJseHlscEVwNlhrQ2Q4Y1BOTDV6MQpreHJ6ZlltUjdS
bm45M2Myd2lrZ0VuUmpNK09IbkVpRHVmdTZJQW16eDE4ODRXcktDZz09Ci0tLS0tRU5EIFJTQSBQ
UklWQVRFIEtFWS0tLS0tCi0tLS0tQkVHSU4gUEdQIFBVQkxJQyBLRVkgQkxPQ0stLS0tLQpWZXJz
aW9uOiBHbnVQRyB2Mi4wLjEzIChHTlUvTGludXgpCgptUUdpQkV2N29Zc1JCQUMrcjE2WGFHWXBJ
Qm1JR09GSDEyMlhOUVk5UVU1clBmQW9EekFydG5JTGhpbVJ1RlFwCkhCc2FyczhHOTdROU1QMU9C
cmx5WG9DTk4rWDJYbEdtU3loUjNnd3RTbVZUaEtudFROWks4K1pIUFl1Z3dHcVMKTjNROU1nTGdS
MnEvWW4vODdrVW5QeWh4bDdtRnhRbmUvMFZKcDFLMzJ4Y0Fld0pyYlduZ0h1QVh6d0NncXhtaQpp
ZncxdlhRQzZXS3k4NEtKUWRrOStzY0QvMmlxa0EwZE8xY2VCUDFtcXdXYVVMMUg1VndZbEhUaytY
NUozQmFPCnREdnR3QUpvMG1GVzQyZnJDYXRWT1FibTdid3NMeEZpZzZ5d1d6NGk4dDR0clh2Rm51
QS8veGhoZGsrRjVEWlIKQko3VVhmcDRWaGhFUGh1dmR0SEN6OC9INlRKTWU0ZHZocGRWblVyVjdm
R1NGdVJuaE5BeFl1U1AvWHJzSVRNMgo2QmFrQS8wV216c1JrUWUyZ0x1blFNdXJCT3NYNkVlUDln
eXoyUHIxZmRTcUxJL3VlRC9hbC9ocTdYd1pwcFNUCkN4Vi9SdUhSdzR5WFczR2RqTnBRWENlczYx
M0UvdG5OamxYMEV4d3FycFRlNFEyelh6d1lLU0RxKzljQ2oxRisKY1ZCVzRSUmxJY0ltc3kzYm9C
c3hlWjBMMEladCt4c252VTh2SGxSTllhQnRQTFFSaXJRc1IzSmxaVzVpYjI1bApJRk5sWTNWeWFY
UjVJRVpsWldRZ1BHWmxaV1JBWjNKbFpXNWliMjVsTG01bGRENklaZ1FURVFJQUpnVUNTL3VoCml3
SWJBd1VKQjRUT0FBWUxDUWdIQXdJRUZRSUlBd1FXQWdNQkFoNEJBaGVBQUFvSkVFQmhPekF3aVB2
R1lHb0EKb0k4cWczcG5CVlh2UGpHVU5lb0tGbWlzV29OQ0FKOS9pMDBQbldGSTVYcUM2a3p5aGpl
dWtNdWZ1Zz09Cj1tVm9iCi0tLS0tRU5EIFBHUCBQVUJMSUMgS0VZIEJMT0NLLS0tLS0K";

    #[test]
    fn test_should_allow_to_validate_feed_key() {
        let mut validator = PlainFeedKeyValidator::new();
        validator.push(VALID_KEY_IDENTIFIER).unwrap();
        validator.push(VALID_PRIVATE_KEY_START).unwrap();
        validator.push(VALID_PRIVATE_KEY_DATA).unwrap();
        validator.push(VALID_PRIVATE_KEY_END).unwrap();
        assert!(validator.done().is_ok());
    }

    #[test]
    fn test_should_fail_if_identifier_is_invalid() {
        let mut validator = PlainFeedKeyValidator::new();
        let result = validator.push("invalid-identifier");
        assert!(result.is_err());
        assert_eq!(result.err().unwrap().to_string(), "Invalid Identifier");
    }

    #[test]
    fn test_should_fail_if_private_key_start_marker_is_missing() {
        let mut validator = PlainFeedKeyValidator::new();
        validator.push(VALID_KEY_IDENTIFIER).unwrap();
        assert_eq!(
            validator.done().err().unwrap().to_string(),
            "Missing Private Key Start marker"
        );
    }

    #[test]
    fn test_should_fail_if_private_key_data_is_invalid() {
        let mut validator = PlainFeedKeyValidator::new();
        validator.push(VALID_KEY_IDENTIFIER).unwrap();
        assert_eq!(
            validator
                .push("invalid-private-key-data")
                .err()
                .unwrap()
                .to_string(),
            "Missing Private Key Start marker"
        );
    }

    #[test]
    fn test_should_fail_if_private_key_data_is_missing() {
        let mut validator = PlainFeedKeyValidator::new();
        validator.push(VALID_KEY_IDENTIFIER).unwrap();
        validator.push(VALID_PRIVATE_KEY_START).unwrap();
        assert_eq!(
            validator.done().err().unwrap().to_string(),
            "Missing Private Key End marker"
        );
    }

    #[test]
    fn test_should_fail_if_private_key_data_is_not_ascii() {
        let mut validator = PlainFeedKeyValidator::new();
        validator.push(VALID_KEY_IDENTIFIER).unwrap();
        validator.push(VALID_PRIVATE_KEY_START).unwrap();
        let result = validator.push("invalid-key-data-ß");
        assert!(result.is_err());
        assert_eq!(result.err().unwrap().to_string(), "Invalid Key data");
    }

    #[test]
    fn test_should_fail_if_private_key_end_marker_is_missing() {
        let mut validator = PlainFeedKeyValidator::new();
        validator.push(VALID_KEY_IDENTIFIER).unwrap();
        validator.push(VALID_PRIVATE_KEY_START).unwrap();
        validator.push(VALID_PRIVATE_KEY_DATA).unwrap();
        assert_eq!(
            validator.done().err().unwrap().to_string(),
            "Missing Private Key End marker"
        );
    }

    #[test]
    fn test_should_allow_to_add_extra_data_after_private_key_end_marker() {
        let mut validator = PlainFeedKeyValidator::new();
        validator.push(VALID_KEY_IDENTIFIER).unwrap();
        validator.push(VALID_PRIVATE_KEY_START).unwrap();
        validator.push(VALID_PRIVATE_KEY_DATA).unwrap();
        validator.push(VALID_PRIVATE_KEY_END).unwrap();
        validator.push("extra-data").unwrap();
        assert!(validator.done().is_ok());
    }

    #[test]
    fn test_should_allow_to_validate_base64_encoded_feed_key() {
        let mut validator = Base64FeedKeyValidator::new();
        for line in OLD_STAGING_KEY.lines() {
            validator.push(line).unwrap();
        }
        assert!(validator.done().is_ok());
    }
}
