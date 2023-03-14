import { useState } from "react"
import { Player } from "./model";

export default function PlayerName(props: {
    assignPlayerName: (name: string) => void,
    playerName: string,
    player: Player,
}) {
    const [pendingPlayerName, setPendingPlayerName] = useState<string | undefined>();

    const assignPlayerName = () => {
        if (pendingPlayerName)
            props.assignPlayerName(pendingPlayerName);
        setPendingPlayerName(undefined);
    }

    return (<form onSubmit={e => { e.preventDefault(); assignPlayerName() }}>
                <label onClick={e => setPendingPlayerName(props.playerName)}>I am:
                    {pendingPlayerName !== undefined ? <input type="text"
                        value={pendingPlayerName}
                        onChange={e => setPendingPlayerName(e.target.value)}
                        onBlur={_ => assignPlayerName()} /> : ` ${props.playerName} `}
                    ({props.player})</label></form>)
}