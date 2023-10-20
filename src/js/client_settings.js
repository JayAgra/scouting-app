async function updateSW() {
    if(window.navigator && navigator.serviceWorker) {
        reg = await navigator.serviceWorker.ready;
        reg.update();
    }
}
function getThemeCookieS() {
    var allcookies = document.cookie.split(';');
    for (var i = 0; i < allcookies.length; i++) {
        var cookie = allcookies[i].trim();
        if ((cookie.indexOf(name)) == 0 && (cookie.substr(name.length)).includes("4c454a5b1bedf6a1")) {
            return cookie.substr(name.length).split('=')[1]
        }
    }
}
function changeTheme() {
    let theme = getThemeCookieS();
    switch (theme) {
        case "light":
            document.cookie = "4c454a5b1bedf6a1=dark; expires=Fri, 31 Dec 9999 23:59:59 GMT; Secure; SameSite=Lax";
            break;
        case "dark": case undefined:
            document.cookie = "4c454a5b1bedf6a1=gruvbox; expires=Fri, 31 Dec 9999 23:59:59 GMT; Secure; SameSite=Lax";
            break;
        case "gruvbox": 
            document.cookie = "4c454a5b1bedf6a1=light; expires=Fri, 31 Dec 9999 23:59:59 GMT; Secure; SameSite=Lax";
            break;
        default:
            document.cookie = "4c454a5b1bedf6a1=dark; expires=Fri, 31 Dec 9999 23:59:59 GMT; Secure; SameSite=Lax";
            break;
    }
    themeHandle()
}