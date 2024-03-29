export type Suit = "Diamonds" | "Clubs" | "Hearts" | "Spades"
export type Rank = "Nine" | "Ten" | "Jack" | "Queen" | "King" | "Ace"
export type Card = { rank: Rank, suit: Suit }
export type Player = "A" | "B" | "C" | "D"

export const nextPlayer = (player: Player): Player => {
    switch (player) {
        case "A": return "B";
        case "B": return "C";
        case "C": return "D";
        case "D": return "A";
    }
}

export const prevPlayer = (player: Player): Player => nextPlayer(nextPlayer(nextPlayer(player)))

export const partner = (player: Player) => nextPlayer(nextPlayer(player));

export const selectionMax = (phase: keyof Phase) => {
    switch (phase) {
        case "Bidding": return 0;
        case "DeclareTrump": return 0;
        case "PassingBack": return 4;
        case "PassingTo": return 4;
        case "RevealingCards": return 12;
        case "ReviewingRevealedCards": return 0;
        case "Play": return 1;
    }
}

export type Phase = {
    "Bidding": {
        first_bidder: Player,
        bids: number[],
    },
    "DeclareTrump": {
        bid_winner: Player,
        highest_bid: number,
    },
    "PassingTo": {
        bid_winner: Player,
        highest_bid: number,
        trump: Suit,
    },
    "PassingBack": {
        bid_winner: Player,
        highest_bid: number,
        trump: Suit,
    },
    "RevealingCards": {
        reveals: ([Suit, Rank][] | null)[],
        extra_points: [number, number],
        bid_winner: Player,
        highest_bid: number,
        trump: Suit,
    },
    "ReviewingRevealedCards": {
        reveals: [Suit, Rank][][],
        reviews: boolean[],
        extra_points: [number, number],
        bid_winner: Player,
        highest_bid: number,
        trump: Suit,
    },
    "Play": {
        trump: Suit,
        bid_winner: Player,
        highest_bid: number,
        extra_points: [number, number],
        piles: [[Suit, Rank][], [Suit, Rank][]],
        trick: {
            "first_player": Player,
            "cards": [Suit, Rank][],
        },
    },
}
export type GameInfo = {
    "player_names": [string, string, string, string],
    "first_bidder": Player,
    "current_player": Player,
    "phase": Phase,
    "scores": [
        number,
        number
    ]
}

export function playerToIndex(player: Player): number {
    return player.charCodeAt(0) - "A".charCodeAt(0)
}


export const toCard = ([suit, rank]: [Suit, Rank]): Card => {
    return { rank, suit };
}

export type FullState = { "seed": number[], "actions": unknown[], "player_names": [string, string, string, string] };

const baseUrl = window.location.port === "3000" ? "http://localhost:8080" : window.location.port === "3001" ? "https://pinochle.sheyne.com" : "";

export class Client {
    async getGames(): Promise<string[]> {
        const result = await fetch(`${baseUrl}/game`, {
            mode: "cors",
        });
        return await result.json()
    }

    async act(game: string, player: Player, action: unknown) {
        const result = await fetch(`${baseUrl}/game/${game}/${player}/act`, {
            mode: "cors",
            method: "POST",
            body: JSON.stringify(action),
            headers: { "Content-Type": "application/json" }
        });
        return await result.text()
    }

    async setName(game: string, player: Player, name: string) {
        const result = await fetch(`${baseUrl}/game/${game}/${player}/name`, {
            mode: "cors",
            method: "PUT",
            body: name,
        });
        return await result.text()
    }

    async startGame(game: string) {
        const result = await fetch(`${baseUrl}/game/${game}`, { mode: "cors", method: "POST" });
        return await result.text()
    }

    async getGameData(game: string): Promise<GameInfo> {
        return await (await fetch(`${baseUrl}/game/${game}`)).json() as GameInfo
    }

    async getHand(game: string, player: Player): Promise<Card[]> {
        const myHandResponse = await fetch(`${baseUrl}/game/${game}/hand/${player}`);
        return (await myHandResponse.json() as [Suit, Rank][]).map(toCard);
    }

    async getFullState(game: string): Promise<FullState> {
        const full = await fetch(`${baseUrl}/game/${game}/full`);
        return await full.json();
    }
    async setFullState(game: string, state: FullState) {
        await fetch(`${baseUrl}/game/${game}`, {
            mode: "cors",
            method: "PUT",
            body: JSON.stringify(state),
            headers: { "Content-Type": "application/json" }
        });
    }
}