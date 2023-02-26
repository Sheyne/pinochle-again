import { useState } from "react"
import "./Hand.css"
import CardView from "./Card"
import { Card, Rank, Suit } from "./model"

const fromFirstSuit = (x: string): Suit | undefined => {
    switch (x[0]) {
        case "D": return "Diamonds"
        case "C": return "Clubs"
        case "H": return "Hearts"
        case "S": return "Spades"
    }
}
const fromFirstRank = (x: string): Rank | undefined => {
    switch (x[0]) {
        case "9":
        case "N": return "Nine"
        case "J": return "Jack"
        case "Q": return "Queen"
        case "K": return "King"
        case "0":
        case "T": return "Ten"
        case "A":
        case "1": return "Ace"
    }
}

function Hand(props: {
    cards: Card[],
    selected?: Set<number>,
    onSelectionChanged?: ((selected: Set<number>) => void),
}) {
    const [focused, setFocused] = useState<undefined | number>(undefined)
    const [suitOrder, setSuitOrder] = useState<string>("DCHS");
    const [rankOrder, setRankOrder] = useState<string>("ATKQJ9");

    const sortCards = (cards: [Card, number][]): [Card, number][] => {
        const ranked = cards.map(([card, index]) =>
            [
                [...suitOrder].map(fromFirstSuit).indexOf(card.suit),
                [...rankOrder].map(fromFirstRank).indexOf(card.rank),
                card, index] as const);
        ranked.sort();
        return ranked.map(([_, _a, card, index]) => [card, index]);
    }

    return (
        <div className="Hand">
            {sortCards(props.cards.map((card, index) => [card, index]))
                .map(([card, index]) =>
                    <CardView
                        card={card}
                        key={index}
                        size="big"
                        selected={props.selected?.has(index) ?? false}
                        focused={index === focused}
                        onFocus={() => setFocused(index)}
                        onUnfocus={() => setFocused(undefined)}
                        onClick={() => {
                            const newSelected = new Set(props.selected);
                            if (newSelected.has(index)) {
                                newSelected.delete(index);
                            } else {
                                newSelected.add(index);
                            }
                            props.onSelectionChanged && props.onSelectionChanged(newSelected)
                        }}
                    />)}
            <div><label>Suit Order:
                <input onChange={(e) => { setSuitOrder(e.target.value) }} value={suitOrder} /></label></div>
            <div><label>Rank Order:
                <input onChange={(e) => { setRankOrder(e.target.value) }} value={rankOrder} /></label></div>
        </div>
    );
}

export default Hand;
