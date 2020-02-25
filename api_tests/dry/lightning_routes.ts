// These are stateless tests -- they don't require any state of the cnd and they don't change it
// They are mostly about checking invalid request responses
import "chai/register-should";
import "../lib/setup_chai";
import { oneActorTest } from "../lib/actor_test";
import { expect } from "chai";

setTimeout(async function() {
    describe("Lightning routes", () => {
        oneActorTest(
            "lightning-routes-post-eth-lnbtc-return-400",
            async function({ alice }) {
                return expect(
                    alice.cnd.postHanEthereumEtherHalightLightningBitcoin()
                ).to.eventually.be.rejected.then(error => {
                    expect(error).to.have.property(
                        "message",
                        "Request failed with status code 400"
                    );
                });
            }
        );

        oneActorTest(
            "lightning-routes-post-erc20-lnbtc-return-400",
            async function({ alice }) {
                return expect(
                    alice.cnd.postHerc20EthereumErc20HalightLightningBitcoin()
                ).to.eventually.be.rejected.then(error => {
                    expect(error).to.have.property(
                        "message",
                        "Request failed with status code 400"
                    );
                });
            }
        );

        oneActorTest(
            "lightning-routes-post-lnbtc-eth-return-400",
            async function({ alice }) {
                return expect(
                    alice.cnd.postHalightLightningBitcoinHanEthereumEther()
                ).to.eventually.be.rejected.then(error => {
                    expect(error).to.have.property(
                        "message",
                        "Request failed with status code 400"
                    );
                });
            }
        );

        oneActorTest(
            "lightning-routes-post-lnbtc-erc20-return-400",
            async function({ alice }) {
                return expect(
                    alice.cnd.postHalightLightningBitcoinHerc20EthereumErc20()
                ).to.eventually.be.rejected.then(error => {
                    expect(error).to.have.property(
                        "message",
                        "Request failed with status code 400"
                    );
                });
            }
        );
    });
    run();
}, 0);
