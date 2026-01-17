interface JWTPayload {
    sub: string;
    email: string;
    role: string | null;
    iat?: number;
    exp?: number;
}
export declare function sign(payload: Omit<JWTPayload, 'iat' | 'exp'>, secret: string): Promise<string>;
export declare function verify(token: string, secret: string): Promise<JWTPayload>;
export {};
//# sourceMappingURL=jwt.d.ts.map