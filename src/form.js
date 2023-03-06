function getThemeCookie() {
    var allcookies = document.cookie.split(';');
    for (var i = 0; i < allcookies.length; i += 1) {
        var cookie = allcookies[i].trim();
        if ((cookie.indexOf(name)) == 0 && (cookie.substr(name.length)).includes("4c454a5b1bedf6a1")) {
            return cookie.substr(name.length).split('=')[1]
        }
    }
}
function themeHandle() {
    let theme = getThemeCookie();
    switch (theme) {
        case "light":
            document.getElementById('body').classList.replace("dark-mode", "light-mode");
            document.getElementById("themeMeta").content = "#ffffff";
            break;
        case "dark": 
        case undefined:
            document.getElementById('body').classList.replace("light-mode", "dark-mode");
            document.getElementById("themeMeta").content = "#121212";
            break;
        default:
            document.getElementById('body').classList.replace("light-mode", "dark-mode");
            document.getElementById("themeMeta").content = "#121212"
    }
}
function goToHome() {
    window.location.href = "/"
}
function exportSubmissionJSON() {
    const downloadFormButton = document.getElementById('downloadFormButton');
    const downloadFileLink = document.getElementById('downloadFileLink');
    downloadFormButton.innerHTML = "Please wait...";
    const form = document.getElementById('mainFormHTML');
    const blob = new Blob([btoa(JSON.stringify(Object.fromEntries(new FormData(form))))], {type: 'application/octet-stream'});
    const downloadURL = URL.createObjectURL(blob);
    downloadFormButton.style.display = "none";
    downloadFileLink.style.display = "inline";
    downloadFileLink.download = "main_submission_" + Date.now() + "_" + Math.random().toString(36).slice(2) + ".scout";
    downloadFileLink.href = downloadURL;
    downloadFileLink.innerHTML = "Ready! Click again to download"
}
function uploadSubmissionJSON() {
    const fileUploadInput = document.getElementById('jsonScoutFile');
    fileUploadInput.disabled = false;
    fileUploadInput.click();
    fileUploadInput.addEventListener('change', async function () {
        let reader = new FileReader();
        await reader.readAsText(fileUploadInput.files[0]);
        reader.onload = () => useFileToFill(reader.result);
        async function useFileToFill(result) {
            console.log(JSON.parse(atob(result)));
            const uploadedData = JSON.parse(atob(result));
            var inputs = Array.prototype.slice.call(document.querySelectorAll('input'));
            var checks = Array.prototype.slice.call(document.querySelectorAll('input[type=checkbox]'));
            var textinputs = Array.prototype.slice.call(document.querySelectorAll('textarea'));
            var selectMenus = Array.prototype.slice.call(document.querySelectorAll('select'));
            Object.keys(uploadedData).map(function (dataItem) {
                    inputs.map(function (inputItem) {return (inputItem.name === dataItem) ? (inputItem.value = uploadedData[dataItem]) : false})
                    checks.map(function (inputItem) {if (uploadedData[dataItem] !== 'true') {inputItem.selected == "true"}})
                    textinputs.map(function (inputItem) {return (inputItem.name === dataItem) ? (inputItem.value = uploadedData[dataItem]) : false})
                    selectMenus.map(function (inputItem) {return (inputItem.name === dataItem) ? (inputItem.value = uploadedData[dataItem]) : false})
            })
        }
    })
}