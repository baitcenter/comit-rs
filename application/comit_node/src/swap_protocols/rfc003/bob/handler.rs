use futures::{
    stream::Stream,
    sync::{mpsc::UnboundedReceiver, oneshot},
    Future,
};
use key_store::KeyStore;
use ledger_query_service::{DefaultLedgerQueryServiceApiClient, FirstMatch, QueryIdCache};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use swap_protocols::{
    asset::Asset,
    metadata_store::{Metadata, MetadataStore},
    rfc003::{
        self,
        bob::SwapRequestKind,
        events::{
            lightning::BobLightningEvents, BobToAlice, CommunicationEvents, LedgerEvents,
            LqsEvents, LqsEventsForErc20,
        },
        roles::Bob,
        state_machine::*,
        state_store::StateStore,
        Ledger,
    },
};
use swaps::common::SwapId;

#[derive(Debug)]
pub struct SwapRequestHandler<MetadataStore, StateStore> {
    // new dependencies
    pub receiver: UnboundedReceiver<(
        SwapId,
        SwapRequestKind,
        oneshot::Sender<rfc003::bob::SwapResponseKind>,
    )>,
    pub metadata_store: Arc<MetadataStore>,
    pub state_store: Arc<StateStore>,
    pub lqs_api_client: Arc<DefaultLedgerQueryServiceApiClient>,
    pub bitcoin_poll_interval: Duration,
    pub ethereum_poll_interval: Duration,
    pub lightning_client_factory: Arc<lightning_rpc::ClientFactory>,
    pub key_store: Arc<KeyStore>,
}

impl<M: MetadataStore<SwapId>, S: StateStore<SwapId>> SwapRequestHandler<M, S> {
    pub fn start(self) -> impl Future<Item = (), Error = ()> {
        let (receiver, metadata_store, bitcoin_poll_interval, ethereum_poll_interval) = (
            self.receiver,
            self.metadata_store,
            self.bitcoin_poll_interval,
            self.ethereum_poll_interval,
        );
        let state_store = Arc::clone(&self.state_store);
        let lqs_api_client = Arc::clone(&self.lqs_api_client);
        let lightning_client_factory = Arc::clone(&self.lightning_client_factory);

        receiver
            .for_each(move |(id, requests, response_sender)| {
                info!("Received swap {} on channel", id);
                match requests {
                    rfc003::bob::SwapRequestKind::BitcoinEthereumBitcoinQuantityEtherQuantity(
                        request,
                    ) => {
                        {
                            let request = request.clone();
                            let metadata: Metadata = request.clone().into();
                            let (bob, response_future) = Bob::new();

                            let response_future = response_future.inspect(|response| {
                                response_sender
                                    .send(response.clone().into())
                                    .expect("receiver should never go out of scope");
                            });

                            let start_state = Start {
                                alpha_ledger_refund_identity: request.alpha_ledger_refund_identity,
                                beta_ledger_success_identity: request.beta_ledger_success_identity,
                                alpha_ledger: request.alpha_ledger,
                                beta_ledger: request.beta_ledger,
                                alpha_asset: request.alpha_asset,
                                beta_asset: request.beta_asset,
                                alpha_ledger_lock_duration: request.alpha_ledger_lock_duration,
                                secret: request.secret_hash,
                                role: bob,
                            };

                            spawn_state_machine(
                                id,
                                start_state,
                                state_store.as_ref(),
                                metadata_store.as_ref(),
                                metadata,
                                Box::new(LqsEvents::new(
                                    QueryIdCache::wrap(Arc::clone(&lqs_api_client)),
                                    FirstMatch::new(
                                        Arc::clone(&lqs_api_client),
                                        bitcoin_poll_interval,
                                    ),
                                )),
                                Box::new(LqsEvents::new(
                                    QueryIdCache::wrap(Arc::clone(&lqs_api_client)),
                                    FirstMatch::new(
                                        Arc::clone(&lqs_api_client),
                                        ethereum_poll_interval,
                                    ),
                                )),
                                Box::new(BobToAlice::new(Box::new(response_future))),
                            );
                        }

                        Ok(())
                    }
                    rfc003::bob::SwapRequestKind::BitcoinEthereumBitcoinQuantityErc20Quantity(
                        request,
                    ) => {
                        {
                            let request = request.clone();
                            let metadata: Metadata = request.clone().into();
                            let (bob, response_future) = Bob::new();

                            let response_future = response_future.inspect(|response| {
                                response_sender
                                    .send(response.clone().into())
                                    .expect("receiver should never go out of scope");
                            });

                            let start_state = Start {
                                alpha_ledger_refund_identity: request.alpha_ledger_refund_identity,
                                beta_ledger_success_identity: request.beta_ledger_success_identity,
                                alpha_ledger: request.alpha_ledger,
                                beta_ledger: request.beta_ledger,
                                alpha_asset: request.alpha_asset,
                                beta_asset: request.beta_asset,
                                alpha_ledger_lock_duration: request.alpha_ledger_lock_duration,
                                secret: request.secret_hash,
                                role: bob,
                            };

                            spawn_state_machine(
                                id,
                                start_state,
                                state_store.as_ref(),
                                metadata_store.as_ref(),
                                metadata,
                                Box::new(LqsEvents::new(
                                    QueryIdCache::wrap(Arc::clone(&lqs_api_client)),
                                    FirstMatch::new(
                                        Arc::clone(&lqs_api_client),
                                        bitcoin_poll_interval,
                                    ),
                                )),
                                Box::new(LqsEventsForErc20::new(
                                    QueryIdCache::wrap(Arc::clone(&lqs_api_client)),
                                    FirstMatch::new(
                                        Arc::clone(&lqs_api_client),
                                        ethereum_poll_interval,
                                    ),
                                )),
                                Box::new(BobToAlice::new(Box::new(response_future))),
                            );
                        }

                        Ok(())
                    }
                    rfc003::bob::SwapRequestKind::EthereumLightningBitcoinQuantityErc20Quantity(
                        request,
                    ) => {
                        let request = request.clone();
                        let metadata: Metadata = request.clone().into();
                        let (bob, response_future) = Bob::new();

                        let response_future = response_future.inspect(|response| {
                            response_sender
                                .send(response.clone().into())
                                .expect("receiver should never go out of scope");
                        });

                        let start_state = Start {
                            alpha_ledger_refund_identity: request.alpha_ledger_refund_identity,
                            beta_ledger_success_identity: request.beta_ledger_success_identity,
                            alpha_ledger: request.alpha_ledger,
                            beta_ledger: request.beta_ledger,
                            alpha_asset: request.alpha_asset,
                            beta_asset: request.beta_asset,
                            alpha_ledger_lock_duration: request.alpha_ledger_lock_duration,
                            secret: request.secret_hash,
                            role: bob,
                        };

                        let state_store = Arc::clone(&state_store);
                        let metadata_store = Arc::clone(&metadata_store);
                        let lqs_api_client = Arc::clone(&lqs_api_client);
                        info!("Attempting to get lnd client for {}", id);
                        tokio::spawn(
                            lightning_client_factory
                                .new_client()
                                .and_then(move |lnd_client| {
                                    info!("Got lnd client for {}", id);
                                    spawn_state_machine(
                                        id,
                                        start_state,
                                        state_store.as_ref(),
                                        metadata_store.as_ref(),
                                        metadata,
                                        Box::new(LqsEventsForErc20::new(
                                            QueryIdCache::wrap(Arc::clone(&lqs_api_client)),
                                            FirstMatch::new(
                                                Arc::clone(&lqs_api_client),
                                                ethereum_poll_interval,
                                            ),
                                        )),
                                        Box::new(BobLightningEvents::new(Arc::new(Mutex::new(
                                            lnd_client,
                                        )))),
                                        Box::new(BobToAlice::new(Box::new(response_future))),
                                    );
                                    Ok(())
                                })
                                .map_err(|e| {
                                    error!("Failed to get lnd client: {:?}", e);
                                    ()
                                }),
                        );
                        Ok(())
                    }
                }
            })
            .map_err(|_| ())
    }
}

fn spawn_state_machine<
    AL: Ledger,
    BL: Ledger,
    AA: Asset,
    BA: Asset,
    S: StateStore<SwapId>,
    M: MetadataStore<SwapId>,
>(
    id: SwapId,
    start_state: Start<Bob<AL, BL, AA, BA>>,
    state_store: &S,
    metadata_store: &M,
    metadata: Metadata,
    alpha_ledger_events: Box<LedgerEvents<AL, AA>>,
    beta_ledger_events: Box<LedgerEvents<BL, BA>>,
    communication_events: Box<CommunicationEvents<Bob<AL, BL, AA, BA>>>,
) {
    let state = SwapStates::Start(start_state);

    metadata_store
        .insert(id, metadata)
        .expect("Failed to store metadata for swap");

    let save_state = state_store
        .insert(id, state.clone())
        .expect("handle errors :)"); // TODO: handle errors

    let context = Context {
        alpha_ledger_events,
        beta_ledger_events,
        state_repo: save_state,
        communication_events,
    };

    info!("Starting state machine for {:?}", id);
    tokio::spawn(
        Swap::start_in(state, context)
            .map(move |outcome| {
                info!("Swap {} finished with {:?}", id, outcome);
            })
            .map_err(move |e| {
                error!("Swap {} failed with {:?}", id, e);
            }),
    );
}
