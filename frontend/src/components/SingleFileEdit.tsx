import axios from 'axios';
import { Buffer } from 'buffer';
import { useEffect, useState } from 'react';
import useSWR, { useSWRConfig } from 'swr';
import { fetcher, serverBaseUrl } from '../global.ts';
import CodeMirror from '@uiw/react-codemirror';
import { Button } from './ui/button.tsx';

interface Props {
  name?: string;
  path: string;
}

export default function SingleFileEdit({ path }: Props) {
  const { mutate } = useSWRConfig();

  let encodedPath = Buffer.from(path).toString('base64');
  const { data, error } = useSWR<{ content: string; path: string }>(`/api/files/${encodedPath}`, fetcher);

  const onUpdate = () => {
    axios
      .put(`${serverBaseUrl}/api/files/${encodedPath}`, {
        content: content,
      })
      .then(function (response) {
        mutate(`/api/files/${encodedPath}`);
      })
      .catch(function (error) {
        console.log(error);
      });
  };

  const [content, setContent] = useState('');

  useEffect(() => {
    setContent(data?.content || '');
  }, [data]);

  if (error) return <div>failed to load</div>;
  if (!data) return <>loading</>;

  return (
    <div>
      <CodeMirror
        value={content}
        height="85vh"
        width="100%"
        onChange={(value) => {
          setContent(value);
        }}
      />
      <Button className="ml-2 mt-2" disabled={data.content === content} onClick={onUpdate}>
        Update
      </Button>
    </div>
  );
}
