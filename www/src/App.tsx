import { useState, useRef } from "react"
import './App.css';
import Controls from "./Controls";
import Hand from "./Hand"
import { Card, GameInfo, Client, Player, selectionMax, Phase } from "./model";


const client = new Client();

const trumpSymbols = {
    "Spades": "♠",
    "Hearts": "♥",
    "Diamonds": "♦",
    "Clubs": "♣",
}

function App() {
    const [selectedCards, setSelectedCards] = useState(new Set<number>());
    const gameCreationElement = useRef<HTMLInputElement | null>(null);
    const [gameName, setGameName] = useState<string | undefined>(undefined);
    const [myHand, setMyHand] = useState<Card[]>([]);
    const [gameData, setGameData] = useState<GameInfo | undefined>(undefined);

    const refresh = async (game: string | undefined = gameName) => {
        if (game) {
            const gameInfo = await client.getGameData(game);
            setGameData(gameInfo);
            const hand = await client.getHand(game, gameInfo.current_player);
            setMyHand(hand);
        }
    }

    const phase = gameData && Object.getOwnPropertyNames(gameData.phase)[0] as keyof Phase

    const gotAction = async (action: unknown) => {
        if (gameName && gameData) {
            await client.act(gameName, gameData.current_player, action);
            setSelectedCards(new Set());
            await refresh();
        }
    }

    const selectCards = (cards: Set<number>) => {
        if (phase) {
            const max = selectionMax(phase);
            if (cards.size <= max) {
                setSelectedCards(cards);
            } else if (max === 1) {
                for (const card of selectedCards) {
                    cards.delete(card);
                }
                if (cards.size === 1) {
                    setSelectedCards(cards);
                }
            }
        }
    }

    const trump = (() => {
        if (gameData) {
            const values = Object.values(gameData.phase)[0];
            if ("trump" in values) {
                return values.trump;
            }
        }
        return undefined;
    })();

    const startGame = async () => {
        const gameName = gameCreationElement.current?.value;
        if (gameName) {
            await client.startGame(gameName);
            setGameName(gameName);
            await refresh(gameName);
        }
    }

    return (
        <div className="App">
            {gameData ? (<div>
                <div>I am: {gameData.current_player}</div>
                <div>A+C: {gameData.scores[0]} B+D: {gameData.scores[1]}</div>
                {trump && <div>Trump is {trumpSymbols[trump]}</div>}
            </div>) : ""}

            {gameData ? <Controls gameInfo={gameData} onAct={gotAction} selectedCards={selectedCards} /> : <div>
                <input type="text" ref={gameCreationElement} />
                <input type="button" value="Create Game" onClick={startGame} />
            </div>}

            <Hand cards={myHand}
                selected={selectedCards}
                onSelectionChanged={selectCards}
            />
        </div>
    );
}

export default App;
