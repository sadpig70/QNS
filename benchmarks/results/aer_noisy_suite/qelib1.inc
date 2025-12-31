// Quantum Experience (QE) Standard Header
gate u3(theta,phi,lambda) q { U(theta,phi,lambda) q; }
gate u2(phi,lambda) q { U(pi/2,phi,lambda) q; }
gate u1(lambda) q { U(0,0,lambda) q; }
gate cx c,t { CX c,t; }
gate id a { U(0,0,0) a; }
gate x a { u3(pi,0,pi) a; }
gate y a { u3(pi,pi/2,pi/2) a; }
gate z a { u1(pi) a; }
gate h a { u2(0,pi) a; }
gate s a { u1(pi/2) a; }
gate sdg a { u1(-pi/2) a; }
gate t a { u1(pi/4) a; }
gate tdg a { u1(-pi/4) a; }
gate rx(theta) a { u3(theta,-pi/2,pi/2) a; }
gate ry(theta) a { u3(theta,0,0) a; }
gate rz(phi) a { u1(phi) a; }
gate cz a,b { h b; cx a,b; h b; }
gate swap a,b { cx a,b; cx b,a; cx a,b; }
