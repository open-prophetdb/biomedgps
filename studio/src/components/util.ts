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
    let redirectUrl = window.location.hash.split("#").pop();
    if (redirectUrl) {
        redirectUrl = redirectUrl.replaceAll('/', '')
        localStorage.setItem('redirectUrl', redirectUrl);
        // Redirect to a warning page that its route name is 'not-authorized'.
        history.push('/not-authorized?redirectUrl=' + redirectUrl);
    } else {
        localStorage.setItem('redirectUrl', '');
        history.push('/not-authorized');
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
