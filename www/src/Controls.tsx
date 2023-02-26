import { useRef } from "react";
import "./Controls.css"
import CardView from "./Card";
import { GameInfo, Phase, Player, partner, toCard, nextPlayer, Suit, Rank } from "./model";

function Controls(props: {
    gameInfo: GameInfo,
    selectedCards: Set<number>,
    player: Player,
    onAct?: (action: unknown) => void,
}) {
    const amCurrentPlayer = props.gameInfo.current_player === props.player;
    const waitingMessage = <div>Waiting for player {props.gameInfo.current_player}</div>
    const onAct = (action: unknown) => {
        if (props.onAct) {
            props.onAct(action)
        }
    }

    const passTo = (target: Player) => {
        if (props.selectedCards.size < 4) {
            return (<div>Select 4 cards to pass to {target}</div>)
        } else {
            return (<input
                type="button"
                value={`Pass to ${target}`}
                onClick={() => onAct({ "Pass": [...props.selectedCards.keys()] })}
            />)
        }
    }

    const displayRevealedCards = (reveals: [Suit, Rank][][]) => {
        let player: Player = "A";
        const cards = reveals.map(x => {
            const res = x.map(toCard).map(x => {
                return (<CardView card={x} label={player} />);
            });
            player = nextPlayer(player);
            return (<div>{res}</div>);
        })
        return <div>{cards}</div>
    }

    const subElements = {
        Bidding: (phase: Phase['Bidding']) => {
            const ref = useRef<HTMLInputElement | null>(null);
            if (!amCurrentPlayer) return waitingMessage;
            return (<form onSubmit={(e) => { e.preventDefault(); onAct({ "Bid": Number(ref.current?.value) }) }}>
                <input type="number" ref={ref} /> <input type="submit" value="Bid" />
            </form>)
        },
        DeclareTrump: (phase: Phase['DeclareTrump']) => {
            if (!amCurrentPlayer) return waitingMessage;
            return (<div>
                Pick a trump suit:
                <input type="button" value="Diamonds" onClick={() => onAct({ "DeclareSuit": "Diamonds" })} />
                <input type="button" value="Clubs" onClick={() => onAct({ "DeclareSuit": "Clubs" })} />
                <input type="button" value="Hearts" onClick={() => onAct({ "DeclareSuit": "Hearts" })} />
                <input type="button" value="Spades" onClick={() => onAct({ "DeclareSuit": "Spades" })} />
            </div>)
        },
        PassingTo: (phase: Phase['PassingTo']) => {
            if (!amCurrentPlayer) return waitingMessage;
            return passTo(phase.bid_winner)
        },
        PassingBack: (phase: Phase['PassingBack']) => {
            if (!amCurrentPlayer) return waitingMessage;
            return passTo(partner(phase.bid_winner))
        },
        RevealingCards: (phase: Phase['RevealingCards']) => {
            const ref = useRef<HTMLInputElement | null>(null);

            return (<div>Reveal cards for points (and manually enter their value)
                {displayRevealedCards(phase.reveals.map(x => x ?? []))}

                {amCurrentPlayer ? (<form onSubmit={(e) => { e.preventDefault(); onAct({ "ShowPoints": [[...props.selectedCards.keys()], Number(ref.current?.value)] }) }}>
                    <input type="number" ref={ref} /> <input type="submit" value="Reveal selected" />
                </form>
                ) : waitingMessage}
            </div>)
        },
        ReviewingRevealedCards: (phase: Phase['ReviewingRevealedCards']) => {
            return (<div>
                {displayRevealedCards(phase.reveals)}
                <input type="button" value="Confirm" onClick={() => onAct("Continue")} />
            </div>)
        },
        Play: (phase: Phase['Play']) => {
            const control = (() => {
                if (!amCurrentPlayer) {
                    return waitingMessage;
                }
                if (props.selectedCards.size === 1) {
                    return (<div><button onClick={() => onAct({ "Play": props.selectedCards.keys().next().value })}>Play</button></div>)
                } else {
                    return (<div>Select a card to play</div>)
                }
            })();

            let playerLabel = phase.trick.first_player;

            return (<div>{control}
                {phase.trick.cards.map(card => {
                    const result = (<CardView card={toCard(card)} focused={true} label={playerLabel} />);
                    playerLabel = nextPlayer(playerLabel);
                    return result;
                })}
            </div>);
        }
    }

    return Object.entries(props.gameInfo.phase).map(([a, b]) => {
        const phase = a as keyof Phase;
        return subElements[phase](b as any);
    })[0];
}

export default Controls;
