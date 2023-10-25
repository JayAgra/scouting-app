function displayErrors() {
    const urlParams = new URLSearchParams(window.location.search);
    const error = urlParams.get("err");
    const errorEl = document.getElementById("error");
    if (error) {
        switch (error) {
            case "0":
                errorEl.innerText = "500 internal server error";
                errorEl.style.display = "unset";
            case "1":
                errorEl.innerText = "bad email/password";
                errorEl.style.display = "unset";
            case "2":
                errorEl.innerText = "account not yet approved by an admin";
                errorEl.style.display = "unset";
            case undefined: default:
                break;
        }
        History.replaceState(null, "", window.location.origin + window.location.pathname);
    }
}