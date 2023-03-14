import { useState, useEffect } from "react"
import { Card, GameInfo, Client, selectionMax, Phase, Player, FullState, playerToIndex } from "./model";
import Controls from "./Controls";
import DevTools from "./DevTools";
import Hand from "./Hand"
import Rules from "./Rules";
import PlayerName from './PlayerName';

const trumpSymbols = {
    "Spades": "♠",
    "Hearts": "♥",
    "Diamonds": "♦",
    "Clubs": "♣",
}


export default function PlayGame({
    myPlayer,
    gameName,
    client
}: {
    myPlayer: Player,
    gameName: string,
    client: Client
}) {
    const [serverFullState, setServerFullState] = useState<FullState | undefined>();
    const [selectedCards, setSelectedCards] = useState(new Set<number>());
    const [myHand, setMyHand] = useState<Card[]>([]);
    const [gameData, setGameData] = useState<GameInfo | undefined>(undefined);
    const [trackFullState, setTrackFullState] = useState<boolean>(false);

    const setFullState = async (newPlayerNames: string, newSeed: string, newActions: string) => {
        try {
            if (gameName) {
                await client.setFullState(gameName, {
                    player_names: (newPlayerNames?.split(",") ?? ["A", "B", "C", "D"]) as [string, string, string, string],
                    seed: JSON.parse(`[${newSeed}]`),
                    actions: JSON.parse(newActions ?? "")
                });
                refresh(gameName);
            }
        } catch { }
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
                setServerFullState(await client.getFullState(game));
            }
        }
    }

    const assignPlayerName = (playerName: string) => {
        if (gameName && myPlayer) {
            client.setName(gameName, myPlayer, playerName);
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

    if (!gameData) {
        return <div>loading...</div>;
    }

    return (
        <div className="App">
            <div>
                <div>
                    <PlayerName
                        assignPlayerName={assignPlayerName}
                        player={myPlayer}
                        playerName={gameData.player_names[playerToIndex(myPlayer)]}
                    />
                     on table: {gameName}</div>
                    <div>
                        {gameData.player_names[0]}+{gameData.player_names[2]} have {gameData.scores[0]} points.
                        {gameData.player_names[1]}+{gameData.player_names[3]} have {gameData.scores[1]} points.
                    </div>
                <Hand cards={myHand}
                    selected={selectedCards}
                    onSelectionChanged={selectCards}
                />
                {trump && <div id="trump-suit"><span className={`suit-${trump}`}>{trumpSymbols[trump]}</span></div>}
                <Controls gameInfo={gameData} player={myPlayer} onAct={gotAction} selectedCards={selectedCards} />
            </div>
            <Rules />
            {
                trackFullState && serverFullState
                    ? <DevTools setFullState={setFullState}
                        actions={JSON.stringify(serverFullState.actions)}
                        playerNames={serverFullState.player_names.toString()}
                        seed={serverFullState.seed.toString()} />
                    : ""}
        </div>
    );
}