import { useState } from "react"
import "./Hand.css"
import CardView from "./Card"
import { Card } from "./model"

function Hand(props: {
    cards: Card[],
    selected?: Set<number>,
    onSelectionChanged?: ((selected: Set<number>) => void),
}) {
    const [focused, setFocused] = useState<undefined | number>(undefined)

    return (
        <div className="Hand">
            {props.cards.map((card, index) =>
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
        </div>
    );
}

export default Hand;
