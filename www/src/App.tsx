import { useState, useRef, useEffect } from "react"
import './App.css';
import Controls from "./Controls";
import Hand from "./Hand"
import { Card, GameInfo, Client, selectionMax, Phase, Player, FullState } from "./model";

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
    const [myPlayer, setMyPlayer] = useState<Player | undefined>(undefined);
    const [trackFullState, setTrackFullState] = useState<boolean>(false);
    const [fullState, setFullState] = useState<FullState>({ seed: [], actions: [] });

    if (!gameName && !gameList) {
        (async () => {
            setGameList(await client.getGames());
        })()
    }

    useEffect(() => {
        const listener = (ev: KeyboardEvent) => {
            if (ev.code === "Backquote" && ev.ctrlKey) {
                setTrackFullState(!trackFullState);
            }
        };
        window.addEventListener("keyup", listener);
        return () => {
            window.removeEventListener("keyup", listener);
        }
    })

    const refresh = async (game: string | undefined = gameName) => {
        if (game) {
            const gameInfo = await client.getGameData(game);
            setGameData(gameInfo);
            if (myPlayer) {
                const hand = await client.getHand(game, myPlayer);
                setMyHand(hand);
            }
            if (trackFullState) {
                setFullState(await client.getFullState(game));
            }
        }
    }

    useEffect(() => {
        const interval = setInterval(refresh, 300);
        return () => clearInterval(interval);
    });

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

            {trackFullState ? <div style={{ backgroundColor: "#ddd", padding: "1em" }}>
                <div><input type="text" style={{ width: "100%" }} value={fullState.seed.toString()} onChange={(e) => {
                    try {
                        if (gameName) {
                            client.setFullState(gameName, { seed: JSON.parse(`[${e.target.value}]`), actions: fullState.actions }); refresh(gameName);
                        }
                    } catch { }
                }} /></div>
                <textarea rows={25} style={{ width: "100%" }} onChange={(e) => {
                    try {
                        if (gameName) {
                            client.setFullState(gameName, { actions: JSON.parse(e.target.value), seed: fullState.seed }); refresh(gameName);
                        }
                    } catch { }
                }} value={JSON.stringify(fullState.actions)}></textarea></div> : ""}
        </div>
    );
}

export default App;
