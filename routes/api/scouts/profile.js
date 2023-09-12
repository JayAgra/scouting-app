function isMe(req) {
    if (req.params.scout === "me") {
        return req.user.id;
    } else {
        return req.params.scout;
    }
}

function profile(req, res, db) {
    const stmt = `SELECT discordID, score, discordProfile, username, discriminator, addedAt, badges FROM scouts WHERE discordID=?`;
    const values = [isMe(req)];
    db.get(stmt, values, (err, dbQueryResult) => {
        if (err) {
            res.status(500).send("got an error from query");
            return;
        } else {
            if (typeof dbQueryResult == "undefined") {
                res.status(204).send("no query results");
            } else {
                res.status(200).json(dbQueryResult);
            }
        }
    });
}

module.exports = { profile };
