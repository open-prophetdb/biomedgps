export const getJwtAccessToken = (): string | null => {
    let jwtToken = null;
    // Check if the cookie exists
    if (document.cookie && document.cookie.includes("jwt_access_token=")) {
        // Retrieve the cookie value
        // @ts-ignore
        jwtToken = document.cookie
            .split("; ")
            .find((row) => row.startsWith("jwt_access_token="))
            .split("=")[1];
    }

    if (jwtToken) {
        console.log("JWT access token found in the cookie.");
        return jwtToken;
    } else {
        console.log("JWT access token not found in the cookie.");
        return null;
    }
}
