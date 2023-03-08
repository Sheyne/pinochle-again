import './Rules.css';
import { useState } from "react"
import Card from './Card';

function Rules() {
    const [showRules, setShowRules] = useState(false);

    return <div>
        <label>Show Rules: <input type="checkbox" checked={showRules} onChange={e=>{setShowRules(e.target.checked)}} /></label>
        {showRules ? <div className="the-rules">
            <h2>The Rules Of Pinochle</h2>
            <h3>Points</h3>
            <table>
                <thead><th></th><th>Example</th><th>Points</th><th>Double</th></thead>
                <tr>
                    <td>Pinochle:</td>
                    <td><Card card={{rank: "Jack", suit: "Diamonds"}}/><Card card={{rank: "Queen", suit: "Spades"}}/></td>
                    <td>40</td>
                    <td>300</td>
                </tr>
                <tr>
                    <td>Marriage:</td>
                    <td><Card card={{rank: "Queen", suit: "Diamonds"}}/><Card card={{rank: "King", suit: "Diamonds"}}/></td>
                    <td>20</td>
                </tr>
                <tr>
                    <td>Trump Marriage:</td>
                    <td><Card card={{rank: "Queen", suit: "Diamonds"}}/><Card card={{rank: "King", suit: "Diamonds"}}/></td>
                    <td>40</td>
                </tr>
                <tr>
                    <td>Round Ace:</td>
                    <td><Card card={{rank: "Ace", suit: "Diamonds"}}/><Card card={{rank: "Ace", suit: "Clubs"}}/><Card card={{rank: "Ace", suit: "Hearts"}}/><Card card={{rank: "Ace", suit: "Spades"}}/></td>
                    <td>100</td>
                </tr>
                <tr>
                    <td>Round King:</td>
                    <td><Card card={{rank: "King", suit: "Diamonds"}}/><Card card={{rank: "King", suit: "Clubs"}}/><Card card={{rank: "King", suit: "Hearts"}}/><Card card={{rank: "King", suit: "Spades"}}/></td>
                    <td>80</td>
                </tr>
                <tr>
                    <td>Round Queen:</td>
                    <td><Card card={{rank: "Queen", suit: "Diamonds"}}/><Card card={{rank: "Queen", suit: "Clubs"}}/><Card card={{rank: "Queen", suit: "Hearts"}}/><Card card={{rank: "Queen", suit: "Spades"}}/></td>
                    <td>60</td>
                </tr>
                <tr>
                    <td>Round Jack:</td>
                    <td><Card card={{rank: "Jack", suit: "Diamonds"}}/><Card card={{rank: "Jack", suit: "Clubs"}}/><Card card={{rank: "Jack", suit: "Hearts"}}/><Card card={{rank: "Jack", suit: "Spades"}}/></td>
                    <td>40</td>
                </tr>
                <tr>
                    <td>Nine of Trump:</td>
                    <td><Card card={{rank: "Nine", suit: "Diamonds"}}/></td>
                    <td>10</td>
                </tr>
                <tr>
                    <td>Run of Trump:</td>
                    <td><Card card={{rank: "Ace", suit: "Diamonds"}}/><Card card={{rank: "Ten", suit: "Diamonds"}}/><Card card={{rank: "King", suit: "Diamonds"}}/><Card card={{rank: "Queen", suit: "Diamonds"}}/><Card card={{rank: "Jack", suit: "Diamonds"}}/></td>
                    <td>150</td>
                    <td>1500</td>
                </tr>
            </table>
            <h3>Notes for passing</h3>
            <p>We generally interpret bidding the smallest amount you can ≤300 as meaning you have a helping hand and can help fill out your partners hand in any suit they mention. We also interpret 275 when you can instead bid 250 as being the same thing, but a little bit more helpful even.</p>

            <p>If your partner is Tamara Descoryphées then you should only bid a helping hand if you have at least 3 parts of Double Pinochle.</p>
            </div>: ""}
        </div>
}

export default Rules;
