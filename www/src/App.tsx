import { useState, useRef } from "react"
import './App.css';
import Controls from "./Controls";
import Hand from "./Hand"
import { Card, GameInfo, Client, selectionMax, Phase, Player } from "./model";

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
    const [gameList, setGameList] = useState<string[] | undefined>(undefined);
    const [gameName, setGameName] = useState<string | undefined>(undefined);
    const [myHand, setMyHand] = useState<Card[]>([]);
    const [gameData, setGameData] = useState<GameInfo | undefined>(undefined);
    const [myPlayer, setMyPlayer] = useState<Player | undefined>(undefined)

    if (!gameName && !gameList) {
        (async () => {
            setGameList(await client.getGames());
        })()
    }

    const refresh = async (game: string | undefined = gameName) => {
        if (game) {
            const gameInfo = await client.getGameData(game);
            setGameData(gameInfo);
            if (myPlayer) {
                const hand = await client.getHand(game, myPlayer);
                setMyHand(hand);
            }
        }
    }

    (window as any).globalRefreshHook = refresh;

    const phase = gameData && Object.getOwnPropertyNames(gameData.phase)[0] as keyof Phase

    const gotAction = async (action: unknown) => {
        if (gameName && gameData && myPlayer) {
            await client.act(gameName, myPlayer, action);
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
            {myPlayer ? (gameData ? (<div>
                <div>I am: {myPlayer} on table: {gameName}</div>
                <div>A+C: {gameData.scores[0]} B+D: {gameData.scores[1]}</div>
                {trump && <div>Trump is {trumpSymbols[trump]}</div>}
                <Controls gameInfo={gameData} player={myPlayer} onAct={gotAction} selectedCards={selectedCards} />
            </div>) : "") : (
                <div>
                    <input type="button" value="Join as player A" onClick={() => setMyPlayer('A')} />
                    <input type="button" value="Join as player B" onClick={() => setMyPlayer('B')} />
                    <input type="button" value="Join as player C" onClick={() => setMyPlayer('C')} />
                    <input type="button" value="Join as player D" onClick={() => setMyPlayer('D')} />
                </div>
            )}

            {gameData ? "" : <div>
                {gameList ? gameList.map(name =>
                    <input type="button" value={`Join "${name}"`} onClick={() => { setGameName(name); refresh(name); }} />) : ""}
                <form onSubmit={e => { e.preventDefault(); startGame(); }}>
                    <input type="text" ref={gameCreationElement} />
                    <input type="submit" value="Create Game" />
                </form>
            </div>}

            <Hand cards={myHand}
                selected={selectedCards}
                onSelectionChanged={selectCards}
            />
        </div>
    );
}

export default App;
