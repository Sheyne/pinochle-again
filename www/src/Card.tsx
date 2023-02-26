import React from "react"
import "./Card.css"
import { Card as CardM } from "./model"

const cardBase = 0x1F0A0;

const rankOffset = (rank: string): number => {
    switch (rank) {
        case "Nine": return 9;
        case "Ten": return 10;
        case "Jack": return 11;
        case "Queen": return 13;
        case "King": return 14;
        case "Ace": return 1;
        default: throw Error("Invalid rank")
    }
}

const suitOffset = (suit: string): number => {
    switch (suit) {
        case "Spades": return 0
        case "Hearts": return 1
        case "Diamonds": return 2
        case "Clubs": return 3
        default: throw Error(`Invalid suit ${suit}`)
    }
}

const getCard = (card: CardM) => {
    return String.fromCodePoint(cardBase + rankOffset(card.rank) + 16 * suitOffset(card.suit))
}

function Card(props: {
    card: CardM,
    selected?: boolean,
    focused?: boolean,
    label?: string,
    size?: "small" | "big",
    onFocus?: React.MouseEventHandler<HTMLSpanElement>,
    onUnfocus?: React.MouseEventHandler<HTMLSpanElement>,
    onClick?: React.MouseEventHandler<HTMLSpanElement>,
}) {
    const size = props.size ?? "small";

    return (
        <span
            className={`Card rank-${props.card.rank} suit-${props.card.suit} focused-${props.focused} selected-${props.selected} size-${size}`}
            onMouseOver={props.onFocus}
            onMouseOut={props.onUnfocus}
            onClick={props.onClick}
        >
            {getCard(props.card)}<span className="label">{props.label ?? ""}</span>
        </span>
    );
}

export default Card;
