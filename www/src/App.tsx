import './App.css';
import { useState, useRef, useEffect } from "react"
import { Card, GameInfo, Client, selectionMax, Phase, Player, FullState, playerToIndex } from "./model";
import Controls from "./Controls";
import Hand from "./Hand"
import Rules from "./Rules";

const client = new Client();

const trumpSymbols = {
    "Spades": "♠",
    "Hearts": "♥",
    "Diamonds": "♦",
    "Clubs": "♣",
}

function App() {
    const [pendingPlayerName, setPendingPlayerName] = useState<string | undefined>();
    const [pendingSeed, setPendingSeed] = useState<string | undefined>();
    const [pendingActions, setPendingActions] = useState<string | undefined>();
    const [pendingPlayerNames, setPendingPlayerNames] = useState<string | undefined>();
    const [lastServerSeed, setLastServerSeed] = useState<string | undefined>();
    const [lastServerActions, setLastServerActions] = useState<string | undefined>();
    const [lastServerPlayerNames, setLastServerPlayerNames] = useState<string | undefined>();
    const [selectedCards, setSelectedCards] = useState(new Set<number>());
    const gameCreationElement = useRef<HTMLInputElement | null>(null);
    const [gameList, setGameList] = useState<string[] | undefined>(undefined);
    const [gameName, setGameName] = useState<string | undefined>(undefined);
    const [myHand, setMyHand] = useState<Card[]>([]);
    const [gameData, setGameData] = useState<GameInfo | undefined>(undefined);
    const [myPlayer, setMyPlayer] = useState<Player | undefined>(undefined);
    const [trackFullState, setTrackFullState] = useState<boolean>(false);

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
                const fullState = await client.getFullState(game);
                const actionsString = JSON.stringify(fullState.actions);
                if (actionsString !== lastServerActions || pendingActions === undefined) {
                    setLastServerActions(actionsString);
                    setPendingActions(actionsString);
                }
                const seedString = fullState.seed.toString();
                if (lastServerSeed != seedString || pendingSeed === undefined) {
                    setLastServerSeed(seedString);
                    setPendingSeed(seedString);
                }
                const playerNamesString = fullState.player_names.toString();
                if (lastServerPlayerNames != playerNamesString || pendingPlayerNames === undefined) {
                    setLastServerPlayerNames(playerNamesString);
                    setPendingPlayerNames(playerNamesString);
                }
            }
        }
    }

    const assignPlayerName = () => {
        if (gameName && myPlayer && pendingPlayerName) {
            client.setName(gameName, myPlayer, pendingPlayerName);
            setPendingPlayerName(undefined);
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
                <div><form onSubmit={e=> {e.preventDefault(); assignPlayerName()}}><label onClick={e => setPendingPlayerName(gameData.player_names[playerToIndex(myPlayer)])}>I am:
                    {pendingPlayerName !== undefined ? <input type="text" 
                        value={pendingPlayerName}
                        onChange={e => setPendingPlayerName(e.target.value)}
                        onBlur={_ => assignPlayerName() } /> : ` ${gameData.player_names[playerToIndex(myPlayer)]} `}
                    ({myPlayer})</label></form> on table: {gameName}</div>
                <div>{gameData.player_names[0]}+{gameData.player_names[2]} have {gameData.scores[0]} points. {gameData.player_names[1]}+{gameData.player_names[3]} have {gameData.scores[1]} points.</div>
                <Hand cards={myHand}
                    selected={selectedCards}
                    onSelectionChanged={selectCards}
                />
                {trump && <div id="trump-suit"><span className={`suit-${trump}`}>{trumpSymbols[trump]}</span></div>}
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

            <Rules />

            {trackFullState ? <form style={{ backgroundColor: "#ddd", padding: "1em" }} onSubmit={(e) => {
                e.preventDefault();
                try {
                    if (gameName) {
                        client.setFullState(gameName, {
                            player_names: (pendingPlayerNames?.split(",") ?? ["A", "B", "C", "D"]) as [string, string, string, string],
                            seed: JSON.parse(`[${pendingSeed}]`),
                            actions: JSON.parse(pendingActions ?? "")
                        }); refresh(gameName);
                    }
                } catch { }
                setPendingActions(undefined);
                setPendingSeed(undefined);
                setPendingPlayerNames(undefined);
            }} >
                <label>PlayerNames: <input type="text" style={{ width: "100%" }} value={pendingPlayerNames} onChange={e => { setPendingPlayerNames(e.target.value); }} /></label>
                <label>Seed: <input type="text" style={{ width: "100%" }} value={pendingSeed} onChange={e => { setPendingSeed(e.target.value); }} /></label>
                <label>Actions:<textarea rows={25} style={{ width: "100%" }} value={pendingActions} onChange={e => { setPendingActions(e.target.value); }}></textarea></label>
                <input type="submit" value="Set" />
            </form> : ""}
        </div>
    );
}

export default App;
