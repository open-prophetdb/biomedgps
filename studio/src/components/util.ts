import { history } from 'umi';

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

export const guessColor = (text: string): string => {
    // TODO: We must change the colors if we add new types or change the existing ones in our database.
    const colors: Record<string, string> = {
        Anatomy: "#1f78b4",
        BiologicalProcess: "#b15928",
        CellularComponent: "#cab2d6",
        Compound: "#33a02c",
        Disease: "#fb9a99",
        Gene: "#e31a1c",
        Metabolite: "#a6cee3",
        MolecularFunction: "#b2df8a",
        Pathway: "#ff7f00",
        PharmacologicClass: "#fdbf6f",
        SideEffect: "#ffff99",
        Symptom: "#6a3d9a"
    }

    return colors[text] || "#108ee9";
}

export const expectedTaxIdOrder: string[] = [
    '9606',
    '10090',
    '10116',
    '9541',
    '9544',
    '9598'
]

export const expectedSpeciesOrder: string[] = [
    'Human',
    'Mouse',
    'Rattus norvegicus',
    'Rat',
    'Macaca fascicularis',
    'M.fascicularis',
    'Macaca mulatta',
    'M.mulatta',
    'Chimpanzee'
]

export const expectedSpecies: Record<string, string[]> = {
    // Full name, abbreviation
    '9606': ['Human', 'Human'],
    '10090': ['Mouse', 'Mouse'],
    '10116': ['Rattus norvegicus', 'Rat'],
    '9541': ['Macaca fascicularis', 'M.fascicularis'],
    '9544': ['Macaca mulatta', 'M.mulatta'],
    '9598': ['Chimpanzee', 'Chimpanzee']
}

export const guessSpecies = (taxid: string) => {
    return expectedSpecies[`${taxid}`] ? expectedSpecies[`${taxid}`][0] : 'Unknown'
}

export const guessSpeciesAbbr = (taxid: string) => {
    return expectedSpecies[`${taxid}`] ? expectedSpecies[`${taxid}`][1] : 'Unknown'
}

export const isExpectedSpecies = (taxid: string) => {
    return expectedSpecies[`${taxid}`] ? true : false
}

export const logout = () => {
    localStorage.removeItem('jwt_access_token');
    localStorage.removeItem('redirectUrl');
}

export const logoutWithRedirect = () => {
    logout();
    // Save the current hash as the redirect url
    let currentUrl = window.location.hash.split("#").pop();
    const baseUrl = '/not-authorized';

    if (currentUrl && !currentUrl.startsWith(baseUrl)) {
        let redirectUrl = currentUrl;

        // 只对非 not-authorized 页面的 URL 进行编码
        redirectUrl = encodeURIComponent(redirectUrl);
        localStorage.setItem('redirectUrl', redirectUrl);
        history.push(`${baseUrl}?redirectUrl=${redirectUrl}`);
    } else {
        // 已在 not-authorized 页面，不再重定向或添加新的 redirectUrl
        localStorage.setItem('redirectUrl', '');
        history.push(baseUrl);
    }
}

export const truncateString = (str: string, num?: number) => {
    if (!num) {
        num = 20;
    }

    if (str.length > num) {
        return str.substring(0, num) + '...';
    } else {
        return str;
    }
}

export const getUsername = (): string | undefined => {
    const accessToken = getJwtAccessToken();
    if (accessToken) {
        try {
            const payload = accessToken.split('.')[1];
            const base64 = payload.replace(/-/g, '+').replace(/_/g, '/');
            const padLength = 4 - (base64.length % 4);
            const paddedBase64 = padLength < 4 ? base64 + "=".repeat(padLength) : base64;
            const payloadJson = JSON.parse(atob(paddedBase64));
            console.log('payloadJson: ', payloadJson);
            return payloadJson?.name || payloadJson?.email || payloadJson?.nickname 
        } catch (error) {
            logout();
            console.log('Error in getUsername: ', error);
            return undefined;
        }
    } else {
        return undefined;
    }
}
