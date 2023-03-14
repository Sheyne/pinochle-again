import './App.css';
import JoinGame from './JoinGame';
import PlayGame from './PlayGame';
import { Client, Player } from "./model";
import { useEffect, useState } from 'react';

const client = new Client();

function getGameStateFromUrl(): { game: string, player: Player } | undefined {
    const urlMatch = window.location.pathname.match(/^\/game\/([^/]*)\/([A-D])$/);
    if (urlMatch) {
        const [, game, player] = urlMatch;
        return { player: player as Player, game };
    } else {
        return undefined;
    }
}

function App() {
    const [activeGame, setActiveGame] = useState<{ game: string, player: Player } | undefined>(getGameStateFromUrl);

    const joinGame = (player: Player, game: string) => {
        setActiveGame({player, game});
        window.history.pushState("", "", `/game/${game}/${player}`);
    }

    useEffect(()=>{
        const listener = () => setActiveGame(getGameStateFromUrl());
        window.addEventListener("popstate", listener);
        return ()=>window.removeEventListener("popstate", listener);
    })

    if (activeGame) {
        return <PlayGame client={client} gameName={activeGame.game} myPlayer={activeGame.player} />
    } else {
        return <JoinGame client={client} onJoinGame={joinGame} />
    }
}

export default App;
