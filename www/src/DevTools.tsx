import { useState } from "react"

export default function DevTools(props: {
    setFullState: (playerNames: string, seed: string, actions: string) => void;
    playerNames: string,
    seed: string,
    actions: string,
}) {
    const [pendingSeed, setPendingSeed] = useState<string | undefined>();
    const [pendingActions, setPendingActions] = useState<string | undefined>();
    const [pendingPlayerNames, setPendingPlayerNames] = useState<string | undefined>();

    const [lastSeed, setLastSeed] = useState<string>(props.seed);
    const [lastActions, setLastActions] = useState<string>(props.actions);
    const [lastPlayerNames, setLastPlayerNames] = useState<string>(props.playerNames);

    if (lastSeed !== props.seed) {
        setLastSeed(props.seed);
        setPendingSeed(undefined);
    }
    if (lastActions !== props.actions) {
        setLastActions(props.actions);
        setPendingActions(undefined);
    }
    if (lastPlayerNames !== props.playerNames) {
        setLastPlayerNames(props.playerNames);
        setPendingPlayerNames(undefined);
    }
    const playerNames = pendingPlayerNames ?? props.playerNames;
    const seed = pendingSeed ?? props.seed;
    const actions = pendingActions ?? props.actions;

    return (
        <form style={{ backgroundColor: "#ddd", padding: "1em" }} onSubmit={(e) => {
            e.preventDefault();
            props.setFullState(playerNames, seed, actions);
            setPendingActions(undefined);
            setPendingSeed(undefined);
            setPendingPlayerNames(undefined);        
        }} >
            <label>PlayerNames: <input type="text" style={{ width: "100%" }} value={playerNames} onChange={e => { setPendingPlayerNames(e.target.value); }} /></label>
            <label>Seed: <input type="text" style={{ width: "100%" }} value={seed} onChange={e => { setPendingSeed(e.target.value); }} /></label>
            <label>Actions:<textarea rows={25} style={{ width: "100%" }} value={actions} onChange={e => { setPendingActions(e.target.value); }}></textarea></label>
            <input type="submit" value="Set" />
        </form>
    )
}