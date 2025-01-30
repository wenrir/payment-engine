//! Integration tests.
#[cfg(test)]
mod tests {
    use crate::{
        engine::run, entities::channel::create_engine_channel,
        entities::transaction::Transaction, entities::EngineEvent,
        filehandler::read_csv,
    };
    macro_rules! test_csv {
        ($fname:expr) => {
            concat!(env!("CARGO_MANIFEST_DIR"), "/src/tests/data/", $fname)
        };
    }
    macro_rules! test_client {
        ($handler:ident, $path:expr, $expected_output: expr) => {
            let (transmit, recv) = create_engine_channel();
            let $handler = tokio::spawn(async move {
                let mut result = vec![];
                let _ = run(recv, &mut result).await;
                assert_eq!(
                    String::from_utf8(result).unwrap(),
                    $expected_output
                );
            });
            let content = read_csv($path);
            assert!(content.is_ok());
            for transaction in content.unwrap().deserialize::<Transaction>() {
                let tx = transaction;
                assert!(tx.is_ok());
                let t = transmit.0.send(EngineEvent::Tx(tx.unwrap())).await; // TODO capture this.
                assert!(t.is_ok());
            }
            let report_res = transmit.0.send(EngineEvent::Report()).await;
            assert!(report_res.is_ok());
            let handler_res = $handler.await;
            assert!(handler_res.is_ok());
        };
    }

    #[tokio::test]
    async fn test_empty_file() {
        let path = test_csv!("empty_file_test.csv");
        test_client!(handler, path, "".to_string());
    }
    #[tokio::test]
    async fn test_precision() {
        let path = test_csv!("precision_test.csv");
        test_client!(handler, path, "client,available,held,total,locked\n1,0.1,0,0.1,false\n2,0.1234,0,0.1234,false\n3,0.1,0,0.1,false\n4,0.02,0,0.02,false\n5,0.1,0,0.1,false\n6,0,0,0,false\n7,0,0,0,false\n".to_string());
    }
    #[tokio::test]
    async fn test_duplicate_tx() {
        let path = test_csv!("duplicate_tx_test.csv");
        test_client!(handler, path, "client,available,held,total,locked\n1,2,0,2,false\n2,0,0,0,false\n".to_string());
    }
    #[tokio::test]
    async fn test_header_only() {
        let path = test_csv!("header_only_test.csv");
        test_client!(handler, path, "".to_string());
    }
    #[tokio::test]
    async fn test_deposit() {
        let path = test_csv!("deposit_test.csv");
        test_client!(handler, path, "client,available,held,total,locked\n1,3,0,3,false\n2,2,0,2,false\n".to_string());
    }
    #[tokio::test]
    async fn test_withdrawl() {
        let path = test_csv!("withdrawl_test.csv");
        test_client!(handler, path, "client,available,held,total,locked\n1,1.5,0,1.5,false\n2,2,0,2,false\n".to_string());
    }
    #[tokio::test]
    async fn test_dispute() {
        let path = test_csv!("dispute_test.csv");
        test_client!(
            handler,
            path,
            "client,available,held,total,locked\n1,0.5,1,1.5,false\n"
                .to_string()
        );
    }
    #[tokio::test]
    async fn test_resolve() {
        let path = test_csv!("resolve_test.csv");
        test_client!(
            handler,
            path,
            "client,available,held,total,locked\n1,1.5,0,1.5,false\n"
                .to_string()
        );
    }
    #[tokio::test]
    async fn test_invalid_resolve() {
        let path = test_csv!("resolve_invalid_test.csv");
        test_client!(
            handler,
            path,
            "client,available,held,total,locked\n1,0.5,1,1.5,false\n"
                .to_string()
        );
    }
    #[tokio::test]
    async fn test_chargeback() {
        let path = test_csv!("chargeback_test.csv");
        test_client!(
            handler,
            path,
            "client,available,held,total,locked\n1,0.5,0,0.5,true\n"
                .to_string()
        );
    }
    #[tokio::test]
    async fn test_invalid_chargeback() {
        let path = test_csv!("chargeback_invalid_test.csv");
        test_client!(
            handler,
            path,
            "client,available,held,total,locked\n1,1.5,0,1.5,false\n"
                .to_string()
        );
    }
}
