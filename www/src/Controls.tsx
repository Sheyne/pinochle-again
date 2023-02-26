import { useRef } from "react";
import "./Controls.css"
import CardView from "./Card";
import { GameInfo, Phase, Player, partner, toCard, nextPlayer } from "./model";

function Controls(props: {
    gameInfo: GameInfo,
    selectedCards: Set<number>,
    player: Player,
    onAct?: (action: unknown) => void,
}) {
    const amCurrentPlayer = props.gameInfo.current_player == props.player;
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

    const subElements = {
        Bidding: (phase: Phase['Bidding']) => {
            const ref = useRef<HTMLInputElement | null>(null);

            return (<form onSubmit={(e) => { e.preventDefault(); onAct({ "Bid": Number(ref.current?.value) }) }}>
                <input type="number" ref={ref} /> <input type="submit" value="Bid" />
            </form>)
        },
        DeclareTrump: (phase: Phase['DeclareTrump']) => {
            return (<div>
                Pick a trump suit:
                <input type="button" value="Diamonds" onClick={() => onAct({ "DeclareSuit": "Diamonds" })} />
                <input type="button" value="Clubs" onClick={() => onAct({ "DeclareSuit": "Clubs" })} />
                <input type="button" value="Hearts" onClick={() => onAct({ "DeclareSuit": "Hearts" })} />
                <input type="button" value="Spades" onClick={() => onAct({ "DeclareSuit": "Spades" })} />
            </div>)
        },
        PassingTo: (phase: Phase['PassingTo']) => {
            return passTo(phase.bid_winner)
        },
        PassingBack: (phase: Phase['PassingBack']) => {
            return passTo(partner(phase.bid_winner))
        },
        RevealingCards: (phase: Phase['RevealingCards']) => {
            const ref = useRef<HTMLInputElement | null>(null);

            return (<div>Reveal cards for points (and manually enter their value)
                <form onSubmit={(e) => { e.preventDefault(); onAct({ "ShowPoints": [[], Number(ref.current?.value)] }) }}>
                    <input type="number" ref={ref} /> <input type="submit" value="Reveal selected" />
                </form>
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

    if (!amCurrentPlayer && !("Play" in props.gameInfo.phase)) {
        return waitingMessage;
    } else {
        return Object.entries(props.gameInfo.phase).map(([a, b]) => {
            const phase = a as keyof Phase;
            return subElements[phase](b as any);
        })[0];
    }
}

export default Controls;
