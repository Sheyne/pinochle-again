import { useRef } from "react";
import "./Controls.css"
import CardView from "./Card";
import { GameInfo, Phase, Player, partner, toCard, Suit, Rank, prevPlayer, Card, playerToIndex } from "./model";

function Controls(props: {
    gameInfo: GameInfo,
    selectedCards: Set<number>,
    player: Player,
    onAct?: (action: unknown) => void,
}) {
    const amCurrentPlayer = props.gameInfo.current_player === props.player;
    const waitingMessage = <div>Waiting for {props.gameInfo.player_names[playerToIndex(props.gameInfo.current_player)]}</div>
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

    const displayPendingPoints = ([ac, bd]: [number, number]) => {
        return <div>Base points {props.gameInfo.player_names[0]}+{props.gameInfo.player_names[2]}: {ac} {props.gameInfo.player_names[1]}+{props.gameInfo.player_names[3]}: {bd}</div>
    }

    const displayRevealedCards = (reveals: [Suit, Rank][][]) => {
        const cards = reveals.map((x, idx) => {
            const res = x.map(toCard).map(x => {
                return (<CardView card={x} label={props.gameInfo.player_names[idx]} />);
            });
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
            return (<div>Reveal cards for points
                {displayRevealedCards(phase.reveals.map(x => x ?? []))}

                {amCurrentPlayer ? (<form onSubmit={(e) => { e.preventDefault(); onAct({ "ShowPoints": [...props.selectedCards.keys()] }) }}>
                    <input type="submit" value="Reveal selected" />
                </form>
                ) : waitingMessage}
            </div>)
        },
        ReviewingRevealedCards: (phase: Phase['ReviewingRevealedCards']) => {
            return (<div>
                {displayPendingPoints(phase.extra_points)}
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

            let firstPlayer = phase.trick.cards.reduce(prevPlayer, props.gameInfo.current_player);
            let lastBidWinner = firstPlayer;
            let winningTeam = (lastBidWinner.codePointAt(0) as number - ("A".codePointAt(0) as number)) % 2
            let lastTrickPile = phase.piles[winningTeam].slice(-4).map(toCard)
            const displayTrickPile = (cards: Card[], label=true) => {
                let playerIndex = playerToIndex(firstPlayer);
                return cards.map(card => {
                    const result = (<CardView card={card} focused={true} label={label ? props.gameInfo.player_names[playerIndex] : undefined} />);
                    playerIndex = (playerIndex + 1) % 4;
                    return result;
                })
            }
            
            
            return (<div>
                {displayPendingPoints(phase.extra_points)}
                {control}
                <div id="this-trick"><h3>This trick:</h3>{displayTrickPile(phase.trick.cards.map(toCard))}</div>
                {phase.trick.cards.length < 2 && lastTrickPile.length > 0 ? <div id="last-trick"><h3>Last trick:</h3>{displayTrickPile(lastTrickPile, false)}</div>: ""}
            </div>);
        }
    }

    return Object.entries(props.gameInfo.phase).map(([a, b]) => {
        const phase = a as keyof Phase;
        return subElements[phase](b as any);
    })[0];
}

export default Controls;
