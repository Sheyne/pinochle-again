import { FormEvent, useEffect, useRef, useState } from "react";
import { Client, Player } from "./model";
import "./JoinGame.css"

export default function JoinGame(props: {client: Client, onJoinGame: (player: Player, game: string) => void}) {
    const [games, setGames] = useState<string[]>([]);
    const newGameName = useRef<HTMLInputElement | null>(null);
    const [pendingGame, setPendingGame] = useState<string | undefined>(undefined);

    const createNewGame = (e: FormEvent<HTMLFormElement>) => {
        e.preventDefault();
        const name = newGameName.current?.value;
        if (name) {
            setPendingGame(name);
            setGames([...games, name].sort());
            props.client.startGame(name);
        }
    }

    const joinAs = (player: Player) => {
        if (pendingGame) {
            props.onJoinGame(player, pendingGame);
        }
    }

    const refresh = async () => {
        setGames((await props.client.getGames()).sort());
    }

    useEffect(() => {
        const interval = setInterval(refresh, 300);
        return () => clearInterval(interval);
    });

    return (
<div>
    <fieldset>
        <legend>Create Game:</legend>
        <form onSubmit={createNewGame}>
        <label>Name: <input ref={newGameName} type="text"/></label>
        <input type="submit" value="Add" />
        </form>
    </fieldset>
    <fieldset>
    <legend>Select Game:</legend>
    {
        games.map(game => 
            <div>
                <label><input type="radio" name="game" value={game} key={game}
                              checked={game === pendingGame}
                              onChange={e => setPendingGame(e.target.value)} />{game}</label>
            </div>
        )
    }
    </fieldset>
    {
        (["A", "B", "C", "D"] as Player[]).map(player => 
            <input type="button" key={player} value={`Join as player ${player}`} onClick={() => joinAs(player)} />
        )
    }
</div>)
}