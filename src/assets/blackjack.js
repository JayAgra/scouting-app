/*jslint browser: true, es6*/
const waitMs = (ms) => new Promise((res) => setTimeout(res, ms));
function goToHome() {
    window.location.href = "/points";
}

const x = Math.min(document.documentElement.clientWidth / 85, document.documentElement.clientHeight / 125);
const styleSheet = `.bjContainer{margin:0;padding:0;}.bjContainer{display:flex;flex-direction:column;align-items:center;justify-content:center;position:fixed;left:-17.5vw;}.blackjack{image-rendering:pixelated;}.cardImg,.handImg{height:${64 * x}px;position:fixed;filter:drop-shadow(0 0 ${x * 2}px #000);}.bjBtn{height:${20 * x}px;position:fixed}.dealer1{top:0;left:${5 * x}px;}.dealer2{top:0;left:${15 * x}px;}.dealer3{top:0;left:${25 * x}px;}.dealer4{top:0;left:${35 * x}px;}.dealer5{top:0;left:${45 * x}px;}.dealer6{top:0;left:${55 * x}px;}.dealer7{top:0;left:${65 * x}px;}.player1{top:${50 * x}px;left:${5 * x}px;}.player2{top:${50 * x}px;left:${15 * x}px;}.player3{top:${50 * x}px;left:${25 * x}px;}.player4{top:${50 * x}px;left:${35 * x}px;}.player5{top:${50 * x}px;left:${45 * x}px;}.player6{top:${50 * x}px;left:${55 * x}px;}.player7{top:${50 * x}px;left:${65 * x}px;}.deal{top:${102 * x}px;left:${-22 * x}px;}.hit{top:${102 * x}px;left:${-2 * x}px;}.stand{top:${102 * x}px;left:${18 * x}px;}.deal.noDeal{display:none;}.hit.noDeal{top:${102 * x}px;left:${46 * x}px;}.stand.noDeal{top:${102 * x}px;left:${18 * x}px;}`;
const stylesheet = new CSSStyleSheet();
stylesheet.replaceSync(styleSheet);
document.adoptedStyleSheets = [stylesheet];

window.disableInputs = false;

var blackjackSocket;

function startBlackjack() {
    setupBoard();

    blackjackSocket = new WebSocket("/api/casino/blackjack/blackjackSocket");

    blackjackSocket.addEventListener("open", () => {
        console.info("blackjack socket opened");
    });

    blackjackSocket.addEventListener("close", () => {
        console.info("blackjack socket closed");
    });

    blackjackSocket.onmessage = async (event) => {
        const data = JSON.parse(event.data);
        console.log(data);

        if (data.card) {
            drawCard(`${document.getElementById("deck").value}card-${data.card.suit}_${data.card.value}.png`, data.target)
            window.disableInputs = false;
        } else if (data.result) {
            alert(data.result);
        }
    };
}

document.getElementsByClassName("hit")[0].onclick = (e) => {
    if (!window.disableInputs) {
        blackjackSocket.send(0x30);
        window.disableInputs = true;
    }
}

document.getElementsByClassName("stand")[0].onclick = (e) => {
    if (!window.disableInputs) {
        blackjackSocket.send(0x31);
        window.disableInputs = true;
    }
}

function setupBoard() {
    // show blackjack
    document.getElementById("start").style.display = "none";
    document.getElementById("game").style.display = "inline";

    document.getElementsByClassName("dealer1")[0].src = `${document.getElementById("deck").value}card_back.png`;
    document.getElementsByClassName("dealer2")[0].src = `${document.getElementById("deck").value}card_back.png`;
    document.getElementsByClassName("player1")[0].src = `${document.getElementById("deck").value}card_back.png`;
    document.getElementsByClassName("player2")[0].src = `${document.getElementById("deck").value}card_back.png`;
}

function drawCard(src, card) {
    document.getElementsByClassName(card)[0].src = src;
}