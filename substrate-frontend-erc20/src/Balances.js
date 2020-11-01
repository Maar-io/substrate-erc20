import React, { useEffect, useState } from 'react';
import { Table, Grid, Button } from 'semantic-ui-react';
import { CopyToClipboard } from 'react-copy-to-clipboard';
import { useSubstrate } from './substrate-lib';
import MaarContract from './MaarToken';

export default function Main (props) {
  const { api, keyring } = useSubstrate();
  const accounts = keyring.getPairs();
  const [balances, setBalances] = useState({});

  const contract = MaarContract(api);

  useEffect(() => {
    const addresses = keyring.getPairs().map(account => account.address);
    let unsubscribeAll = null;

    const getBalanceOf = async (account) => {
      const balance = await contract.query.balanceOf(account, 0, -1, account);
      return balance;
    };

    const allBalances = addresses.map((address) => {
      return getBalanceOf(address).then(result => ({ ...result, address }));
    });

    Promise.all(allBalances).then(balances => {
      const balancesMap = balances.reduce((acc, { result, address, output }, index) => {
        if (result.isSuccess) {
          return {
            ...acc,
            [address]: output.toHuman()
          };
        }
        return acc;
      }, {});
      setBalances(balancesMap);
    }).then(sub => {
      unsubscribeAll = sub;
    }).catch(console.error);

    return () => unsubscribeAll && unsubscribeAll();
  }, [api, keyring, setBalances, contract]);

  return (
    <Grid.Column>
      <h1>MAAR Token Balances</h1>
      <Table celled striped size='small'>
        <Table.Body>{accounts.map(account =>
          <Table.Row key={account.address}>
            <Table.Cell width={3} textAlign='right'>{account.meta.name}</Table.Cell>
            <Table.Cell width={10}>
              <span style={{ display: 'inline-block', minWidth: '31em' }}>
                {account.address}
              </span>
              <CopyToClipboard text={account.address}>
                <Button
                  basic
                  circular
                  compact
                  size='mini'
                  color='blue'
                  icon='copy outline'
                />
              </CopyToClipboard>
            </Table.Cell>
            <Table.Cell width={3}>{
              balances && balances[account.address] &&
              balances[account.address]
            }</Table.Cell>
          </Table.Row>
        )}
        </Table.Body>
      </Table>
    </Grid.Column>
  );
}
